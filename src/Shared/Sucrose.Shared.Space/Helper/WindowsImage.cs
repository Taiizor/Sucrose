using Microsoft.Win32;
using System.Buffers.Binary;
using System.Globalization;
using System.IO;
using System.Numerics;
using System.Runtime.InteropServices;

namespace Sucrose.Shared.Space.Helper
{
    internal static class WindowsImage
    {
        public const uint DefaultFeatureId = 38943831;

        // Builds before 25H2 do not gate lock-screen animation behind this feature.
        public const int MinimumCheckedBuild = 26200;

        public static bool IsCheckRequired()
        {
            try
            {
                using RegistryKey Machine = RegistryKey.OpenBaseKey(RegistryHive.LocalMachine, RegistryView.Registry64);
                using RegistryKey Current = Machine.OpenSubKey(@"SOFTWARE\Microsoft\Windows NT\CurrentVersion");

                object Value = Current?.GetValue("CurrentBuildNumber");

                if (Value is not null && int.TryParse(Value.ToString(), out int Build))
                {
                    return Build >= MinimumCheckedBuild;
                }
            }
            catch
            {
            }

            return false;
        }

        public static string GetOverridePath(uint FeatureId)
        {
            if (FeatureId == 0)
            {
                throw new ArgumentOutOfRangeException(nameof(FeatureId));
            }

            // Windows encodes the numeric ID in its registry leaf.
            uint Reversed = BinaryPrimitives.ReverseEndianness(FeatureId ^ 0x74161A4Eu);
            uint Encoded = BitOperations.RotateLeft(Reversed ^ 0x8FB23D4Fu, 1) ^ 0x833EA8FFu;

            return @"SYSTEM\CurrentControlSet\Control\FeatureManagement\Overrides\8\" + Encoded.ToString(CultureInfo.InvariantCulture);
        }

        public static FeatureState Read()
        {
            return Read(DefaultFeatureId);
        }

        public static FeatureState Read(uint FeatureId)
        {
            if (FeatureId == 0)
            {
                throw new ArgumentOutOfRangeException(nameof(FeatureId));
            }

            FeatureState State = new() { FeatureId = FeatureId, ObservedAt = DateTimeOffset.UtcNow };

            try
            {
                ulong Stamp = 0;
                State.QueryStatus = RtlQueryFeatureConfiguration(FeatureId, 1, ref Stamp, out FeatureConfiguration Configuration);

                if (State.QueryStatus == 0)
                {
                    if (Configuration.FeatureId != FeatureId)
                    {
                        State.QueryError = "Windows returned a different feature identifier.";
                    }
                    else
                    {
                        State.RuntimeState = (Configuration.Flags >> 4) & 3;
                        State.RuntimePriority = Configuration.Flags & 15;
                    }
                }
            }
            catch (Exception Exception)
            {
                State.QueryError = $"{Exception.GetType().Name} (0x{unchecked((uint)Exception.HResult):X8}): {Exception.Message}";
            }

            ReadOverride(State);

            return State;
        }

        public static bool IsEnabled(FeatureState State)
        {
            return State.QueryStatus == 0 && State.QueryError is null && State.RuntimeState == 2;
        }

        public static bool IsDisabled(FeatureState State)
        {
            return State.QueryStatus == 0 && State.QueryError is null && State.RuntimeState == 1;
        }

        public static string EnsureDisabled()
        {
            return EnsureState(false);
        }

        public static string EnsureEnabled()
        {
            return EnsureState(true);
        }

        private static string EnsureState(bool Enabled)
        {
            uint Desired = Enabled ? 2u : 1u;

            FeatureState Before = Read();

            if (Before.QueryStatus != 0 || Before.QueryError is not null || Before.RuntimeState is not (1 or 2))
            {
                return $"ERR|{Before.QueryError ?? "Windows did not return an explicit enabled or disabled runtime configuration."}";
            }

            if (Before.RuntimePriority > 8 && Before.RuntimeState != Desired)
            {
                return "ERR|A higher-priority Windows configuration conflicts with the requested state.";
            }

            if (Before.RuntimeState == Desired && Before.OverrideState != (Enabled ? 1 : 2))
            {
                return "OK|AlreadyDisabled";
            }

            bool RuntimeAttempted = false;
            string Error = null;

            try
            {
                if (Before.RuntimeState != Desired)
                {
                    RuntimeAttempted = true;

                    int Status = SetRuntime(Before.FeatureId, Desired);

                    if (Status != 0)
                    {
                        Error = $"The Windows runtime feature update failed with NTSTATUS 0x{unchecked((uint)Status):X8}.";
                    }
                }

                if (Error is null && (Before.OverrideState != Desired || Before.OverrideOptions != 0))
                {
                    WriteOverride(Before, Desired);
                }
            }
            catch (Exception Exception)
            {
                Error = $"{Exception.GetType().Name} (0x{unchecked((uint)Exception.HResult):X8}): {Exception.Message}";
            }

            FeatureState After;

            try
            {
                After = Read();
            }
            catch (Exception Exception)
            {
                return $"ERR|Verification failed: {Exception.Message}";
            }

            bool RuntimeReadable = After.QueryStatus == 0 && After.QueryError is null && After.RuntimeState is 1 or 2;
            bool PersistenceReadable = After.OverrideError is null && After.OverrideExists is not null;

            if (!RuntimeReadable || After.RuntimeState != Desired || !PersistenceReadable || After.OverrideState != Desired || After.OverrideOptions != 0)
            {
                string Detail = Error ?? After.QueryError ?? After.OverrideError ?? "The requested runtime state and persistent override could not both be verified.";
                _ = RuntimeAttempted;
                return $"ERR|{Detail}";
            }

            if (Error is not null)
            {
                return $"ERR|{Error}";
            }

            return "OK|Disabled";
        }

        private static int SetRuntime(uint FeatureId, uint Desired)
        {
            ulong Stamp = 0;
            FeatureUpdate[] Updates =
            [
                new()
                {
                    FeatureId = FeatureId,
                    Priority = 8,
                    EnabledState = Desired,
                    EnabledStateOptions = 0,
                    Variant = 0,
                    VariantPayloadKind = 0,
                    VariantPayload = 0,
                    Operation = 1
                }
            ];

            return RtlSetFeatureConfigurations(ref Stamp, 1, Updates, Updates.Length);
        }

        private static void WriteOverride(FeatureState Expected, uint Desired)
        {
            using RegistryKey Machine = RegistryKey.OpenBaseKey(RegistryHive.LocalMachine, RegistryView.Registry64);

            string OverridePath = GetOverridePath(Expected.FeatureId);

            using RegistryKey Existing = Machine.OpenSubKey(OverridePath, true);

            if ((Existing is not null) != Expected.OverrideExists)
            {
                throw new IOException("The pending feature override changed after it was checked. Retry the feature action.");
            }

            using RegistryKey Created = Existing is null ? Machine.CreateSubKey(OverridePath, true) : null;

            RegistryKey Target = Existing ?? Created ?? throw new IOException("The feature override could not be opened for writing.");

            int? CurrentState = ReadDword(Target, "EnabledState", 2);
            int? CurrentOptions = ReadDword(Target, "EnabledStateOptions", 1);

            if (CurrentState != Expected.OverrideState || CurrentOptions != Expected.OverrideOptions)
            {
                throw new IOException("The pending feature override changed while it was being opened. Retry the feature action.");
            }

            // Preserve every unrelated value Windows maintains under the leaf.
            Target.SetValue("EnabledState", (int)Desired, RegistryValueKind.DWord);
            Target.SetValue("EnabledStateOptions", 0, RegistryValueKind.DWord);
            Target.Flush();
        }

        private static void ReadOverride(FeatureState State)
        {
            try
            {
                using RegistryKey Machine = RegistryKey.OpenBaseKey(RegistryHive.LocalMachine, RegistryView.Registry64);
                using RegistryKey Key = Machine.OpenSubKey(GetOverridePath(State.FeatureId));

                State.OverrideExists = Key is not null;

                if (Key is null)
                {
                    return;
                }

                State.OverrideState = ReadDword(Key, "EnabledState", 2);
                State.OverrideOptions = ReadDword(Key, "EnabledStateOptions", 1);
            }
            catch (Exception Exception)
            {
                State.OverrideError = $"{Exception.GetType().Name} (0x{unchecked((uint)Exception.HResult):X8}): {Exception.Message}";
            }
        }

        private static int? ReadDword(RegistryKey Key, string Name, int Maximum)
        {
            object Value = Key.GetValue(Name, null, RegistryValueOptions.DoNotExpandEnvironmentNames);

            if (Value is null)
            {
                return null;
            }

            if (Key.GetValueKind(Name) != RegistryValueKind.DWord || Value is not int Number || Number < 0 || Number > Maximum)
            {
                throw new InvalidDataException($"The feature override value {Name} has an unexpected type or value.");
            }

            return Number;
        }

        internal sealed class FeatureState
        {
            public uint FeatureId { get; set; }
            public DateTimeOffset? ObservedAt { get; set; }
            public int? QueryStatus { get; set; }
            public string QueryError { get; set; }
            public uint? RuntimeState { get; set; }
            public uint? RuntimePriority { get; set; }
            public bool? OverrideExists { get; set; }
            public int? OverrideState { get; set; }
            public int? OverrideOptions { get; set; }
            public string OverrideError { get; set; }
        }

        [StructLayout(LayoutKind.Sequential)]
        private struct FeatureConfiguration
        {
            internal uint FeatureId;
            internal uint Flags;
            internal uint VariantPayload;
        }

        [StructLayout(LayoutKind.Sequential)]
        private struct FeatureUpdate
        {
            internal uint FeatureId;
            internal uint Priority;
            internal uint EnabledState;
            internal uint EnabledStateOptions;
            internal uint Variant;
            internal uint VariantPayloadKind;
            internal uint VariantPayload;
            internal uint Operation;
        }

        [DllImport("ntdll.dll", ExactSpelling = true)]
        [DefaultDllImportSearchPaths(DllImportSearchPath.System32)]
        private static extern int RtlQueryFeatureConfiguration(uint FeatureId, uint ConfigurationType, ref ulong ChangeStamp, out FeatureConfiguration Configuration);

        [DllImport("ntdll.dll", ExactSpelling = true)]
        [DefaultDllImportSearchPaths(DllImportSearchPath.System32)]
        private static extern int RtlSetFeatureConfigurations(ref ulong PreviousChangeStamp, uint ConfigurationType, [In] FeatureUpdate[] Updates, int Count);
    }
}
