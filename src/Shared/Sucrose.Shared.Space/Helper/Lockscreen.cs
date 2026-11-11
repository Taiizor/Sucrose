using Microsoft.Win32;
using System.Diagnostics;
using System.IO;
using System.IO.Compression;
using System.Runtime.InteropServices;
using System.Security.AccessControl;
using System.Security.Cryptography;
using System.Security.Principal;
using System.Text.RegularExpressions;

namespace Sucrose.Shared.Space.Helper
{
    internal static class Lockscreen
    {
        public const string MainImageName = "LockScreen.jpg";

        public const string VariantSuffix = "_notdimmed.jpg";

        public enum LockScreenMode
        {
            PictureOrOther,
            Slideshow,
            Spotlight,
            Unknown
        }

        public static string GetCacheDirectory()
        {
            using WindowsIdentity Identity = WindowsIdentity.GetCurrent();

            string Sid = Identity.User?.Value ?? throw new InvalidOperationException("Unable to obtain the current user SID.");

            return Path.Combine(
                Environment.GetFolderPath(Environment.SpecialFolder.CommonApplicationData),
                "Microsoft",
                "Windows",
                "SystemData",
                Sid,
                "ReadOnly"
            );
        }

        public static string GetRoot(string CacheDirectory)
        {
            return Path.GetFullPath(CacheDirectory).TrimEnd(Path.DirectorySeparatorChar);
        }

        public static bool IsAdministrator()
        {
            using WindowsIdentity Identity = WindowsIdentity.GetCurrent();

            return new WindowsPrincipal(Identity).IsInRole(WindowsBuiltInRole.Administrator);
        }

        public static LockScreenMode TryGetLockScreenMode()
        {
            try
            {
                using RegistryKey Settings = Registry.CurrentUser.OpenSubKey(@"Software\Microsoft\Windows\CurrentVersion\Lock Screen");
                using RegistryKey Delivery = Registry.CurrentUser.OpenSubKey(@"Software\Microsoft\Windows\CurrentVersion\ContentDeliveryManager");

                object Slideshow = Settings?.GetValue("SlideshowEnabled");
                object Spotlight = Delivery?.GetValue("RotatingLockScreenEnabled");

                bool SlideshowMissing = Settings is not null && Slideshow is null && !Settings.GetValueNames().Contains("SlideshowEnabled", StringComparer.OrdinalIgnoreCase);

                return InferMode(Slideshow, Spotlight, SlideshowMissing);
            }
            catch
            {
                return LockScreenMode.Unknown;
            }
        }

        internal static LockScreenMode InferMode(object Slideshow, object Spotlight, bool SlideshowMissing = false)
        {
            bool? Slide = ReadSwitch(Slideshow);
            bool? Rotate = ReadSwitch(Spotlight);

            if (Slide == true && Rotate == true)
            {
                return LockScreenMode.Unknown;
            }

            if (Slide == true)
            {
                return LockScreenMode.Slideshow;
            }

            if (Rotate == true)
            {
                return LockScreenMode.Spotlight;
            }

            return (Slide == false || (Slideshow is null && SlideshowMissing)) && Rotate == false
                ? LockScreenMode.PictureOrOther
                : LockScreenMode.Unknown;
        }

        private static bool? ReadSwitch(object Value)
        {
            return Value switch
            {
                int Number when Number is 0 or 1 => Number == 1,
                byte[] Bytes when Bytes.Length == 1 && Bytes[0] is 0 or 1 => Bytes[0] == 1,
                _ => null,
            };
        }

        public static bool IsGif(string Path)
        {
            try
            {
                using FileStream Stream = new(Path, FileMode.Open, FileAccess.Read, FileShare.Read);

                byte[] Header = new byte[6];

                if (Stream.Read(Header, 0, 6) != 6)
                {
                    return false;
                }

                string Signature = System.Text.Encoding.ASCII.GetString(Header);

                return Signature is "GIF87a" or "GIF89a";
            }
            catch
            {
                return false;
            }
        }

        public static string HashFile(string Path)
        {
            using FileStream Stream = new(Path, FileMode.Open, FileAccess.Read, FileShare.Read);

            return Convert.ToHexString(SHA256.HashData(Stream));
        }

        public static bool IsAccessDenied(Exception Exception)
        {
            return Exception is UnauthorizedAccessException || Exception.HResult == unchecked((int)0x80070005);
        }

        public static void ValidatePath(string Root, string Path)
        {
            ValidatePath(Root, Path, false);
        }

        private static void ValidatePath(string Root, string Path, bool DeferDeniedAttributes)
        {
            string Full = System.IO.Path.GetFullPath(Path);

            if (!Full.Equals(Root, StringComparison.OrdinalIgnoreCase) && !Full.StartsWith(Root + System.IO.Path.DirectorySeparatorChar, StringComparison.OrdinalIgnoreCase))
            {
                throw new InvalidOperationException("The operation is outside the current user lock-screen cache.");
            }

            for (string Current = Full; Current is not null; Current = System.IO.Path.GetDirectoryName(Current))
            {
                try
                {
                    if ((File.GetAttributes(Current) & FileAttributes.ReparsePoint) != 0)
                    {
                        throw new InvalidOperationException("Reparse points are not supported in the lock-screen cache.");
                    }
                }
                catch (FileNotFoundException)
                {
                }
                catch (DirectoryNotFoundException)
                {
                }
                catch (Exception Exception) when (IsAccessDenied(Exception))
                {
                    if (!DeferDeniedAttributes)
                    {
                        throw;
                    }
                    // A repair request may proceed: the privileged side re-checks
                    // every ancestor strictly once attributes are readable again.
                }
            }
        }

        public static string[] FindCacheFolders(string CacheDirectory)
        {
            string Root = GetRoot(CacheDirectory);

            ValidatePath(Root, CacheDirectory);

            if (!Directory.Exists(CacheDirectory))
            {
                return Array.Empty<string>();
            }

            return Directory
                .GetDirectories(CacheDirectory)
                .Where(Path => System.IO.Path.GetFileName(Path).StartsWith("LockScreen", StringComparison.OrdinalIgnoreCase))
                .Order(StringComparer.OrdinalIgnoreCase)
                .ToArray();
        }

        public static List<string> FindDestinations(string CacheDirectory)
        {
            string[] Folders = FindCacheFolders(CacheDirectory);
            string[] Resolutions = GetDisplayResolutions();

            HashSet<string> Destinations = new(StringComparer.OrdinalIgnoreCase);

            foreach (string Folder in Folders)
            {
                Destinations.Add(System.IO.Path.Combine(Folder, MainImageName));

                foreach (string Resolution in Resolutions)
                {
                    Destinations.Add(System.IO.Path.Combine(Folder, $"LockScreen___{Resolution}{VariantSuffix}"));
                }

                string[] Existing = Array.Empty<string>();

                try
                {
                    Existing = Directory.GetFiles(Folder, "*" + VariantSuffix);
                }
                catch
                {
                }

                foreach (string Variant in Existing)
                {
                    Destinations.Add(Variant);
                }
            }

            return Destinations.Order(StringComparer.OrdinalIgnoreCase).ToList();
        }

        public static string[] GetDisplayResolutions()
        {
            try
            {
                List<string> Resolutions = new();

                DISPLAY_DEVICE Device = new();
                Device.cb = Marshal.SizeOf<DISPLAY_DEVICE>();

                for (uint Index = 0; EnumDisplayDevices(null, Index, ref Device, 0); Index++)
                {
                    if ((Device.StateFlags & 0x1) == 0)
                    {
                        continue;
                    }

                    DEVMODE Mode = new();
                    Mode.dmSize = (short)Marshal.SizeOf<DEVMODE>();

                    if (EnumDisplaySettings(Device.DeviceName, -1, ref Mode))
                    {
                        Resolutions.Add($"{Mode.dmPelsWidth:0000}_{Mode.dmPelsHeight:0000}");
                    }

                    Device.cb = Marshal.SizeOf<DISPLAY_DEVICE>();
                }

                return Resolutions.Distinct(StringComparer.OrdinalIgnoreCase).ToArray();
            }
            catch
            {
                return Array.Empty<string>();
            }
        }

        private static async Task<int> RunToolAsync(string Name, params string[] Args)
        {
            ProcessStartInfo Start = new(System.IO.Path.Combine(Environment.SystemDirectory, Name))
            {
                UseShellExecute = false,
                CreateNoWindow = true,
                RedirectStandardOutput = true,
                RedirectStandardError = true
            };

            foreach (string Arg in Args)
            {
                Start.ArgumentList.Add(Arg);
            }

            using Process Process = Process.Start(Start) ?? throw new IOException("Access tool did not start.");

            Task<string> Output = Process.StandardOutput.ReadToEndAsync();
            Task<string> Error = Process.StandardError.ReadToEndAsync();

            await Process.WaitForExitAsync();
            await Task.WhenAll(Output, Error);

            return Process.ExitCode;
        }

        public static async Task GrantAsync(string CacheDirectory, string Path, bool Write)
        {
            string Root = GetRoot(CacheDirectory);

            // Lexical scope plus visible links first. Unreadable attributes are
            // re-checked strictly after the repair, when they must be readable.
            ValidatePath(Root, Path, true);

            using WindowsIdentity Identity = WindowsIdentity.GetCurrent();

            string Sid = Identity.User?.Value ?? throw new InvalidOperationException("Unable to obtain the current user SID.");

            // icacls itself may need readable attributes on the SID parent even
            // after takeown. Permit metadata reads there without granting any
            // listing, file-data or write rights (mirrors the reference helper).
            await EnsureParentMetadataReadableAsync(Root, Sid, Path);
            ValidatePath(Root, Path, true);

            string Rights = Write ? "M" : "RX";

            await RepairPathAsync(Root, Path, Sid, Rights);

            // A repair reply is not permission to skip validation. The scope may
            // still be unreadable, and paths can change while the tools run.
            ValidatePath(Root, Path, false);
        }

        private static async Task EnsureParentMetadataReadableAsync(string Root, string Sid, string Path)
        {
            string Parent = System.IO.Path.GetDirectoryName(Root) ?? throw new ArgumentException("The cache must have a user parent directory.");

            try
            {
                GrantParentMetadata(Parent, Sid);
            }
            catch (UnauthorizedAccessException)
            {
                ValidatePath(Root, Path, true);

                int Exit = await RunToolAsync("takeown.exe", "/f", Parent, "/a");

                if (Exit != 0)
                {
                    throw new UnauthorizedAccessException($"Cache permission repair failed with exit code {Exit}.");
                }

                ValidatePath(Root, Path, true);

                GrantParentMetadata(Parent, Sid);
            }
        }

        private static void GrantParentMetadata(string Parent, string Sid)
        {
            DirectoryInfo Info = new(Parent);
            DirectorySecurity Access = Info.GetAccessControl(AccessControlSections.Access);
            Access.AddAccessRule(
                new FileSystemAccessRule(
                    new SecurityIdentifier(Sid),
                    FileSystemRights.ReadAttributes,
                    AccessControlType.Allow));
            Info.SetAccessControl(Access);
        }

        private static async Task RepairPathAsync(string Root, string Path, string Sid, string Rights)
        {
            string Grant = $"*{Sid}:{Rights}";

            int Exit = await RunToolAsync("icacls.exe", Path, "/grant", Grant);

            if (Exit == 0)
            {
                return;
            }

            ValidatePath(Root, Path, true);

            Exit = await RunToolAsync("takeown.exe", "/f", Path, "/a");

            if (Exit != 0)
            {
                throw new UnauthorizedAccessException($"Cache permission repair failed with exit code {Exit}.");
            }

            ValidatePath(Root, Path, true);

            Exit = await RunToolAsync("icacls.exe", Path, "/grant", Grant);

            if (Exit != 0)
            {
                throw new UnauthorizedAccessException($"Cache permission repair failed with exit code {Exit}.");
            }
        }

        public static async Task<(int Succeeded, int Failed, string Error)> ApplyGifAsync(string CacheDirectory, string SourcePath)
        {
            if (!File.Exists(SourcePath))
            {
                return (0, 0, "The selected GIF could not be found.");
            }

            if (!IsGif(SourcePath))
            {
                return (0, 0, "The selected file is not a readable GIF.");
            }

            string Root = GetRoot(CacheDirectory);
            string Hash = HashFile(SourcePath);

            List<string> Destinations;

            try
            {
                Destinations = FindDestinations(CacheDirectory);
            }
            catch (Exception Exception) when (IsAccessDenied(Exception))
            {
                await GrantAsync(CacheDirectory, CacheDirectory, false);

                Destinations = FindDestinations(CacheDirectory);
            }

            if (Destinations.Count == 0)
            {
                return (0, 0, "No lock-screen cache destinations were found. Choose Picture in Windows lock-screen settings and retry.");
            }

            int Succeeded = 0;
            int Failed = 0;
            Exception Failure = null;

            foreach (string Destination in Destinations)
            {
                try
                {
                    await WriteVerifiedAsync(Root, CacheDirectory, SourcePath, Hash, Destination);

                    Succeeded++;
                }
                catch (Exception Exception)
                {
                    Failed++;
                    Failure ??= Exception;
                }
            }

            if (Failed > 0 && Succeeded == 0)
            {
                return (0, Failed, DescribeRecovery(Failure));
            }

            if (Failed > 0)
            {
                return (Succeeded, Failed, $"Only {Succeeded} of {Destinations.Count} destinations were verified. {DescribeRecovery(Failure)}");
            }

            return (Succeeded, 0, null);
        }

        private static async Task WriteVerifiedAsync(string Root, string CacheDirectory, string SourcePath, string Hash, string Destination)
        {
            ValidatePath(Root, Destination);

            string Parent = System.IO.Path.GetDirectoryName(Destination)!;
            string Staged = System.IO.Path.Combine(
                Parent,
                $".{System.IO.Path.GetFileName(Destination)}.{Guid.NewGuid():N}.lockscreen.tmp"
            );

            ValidatePath(Root, Staged);

            try
            {
                // Staging creates a new file, so the parent folder must be
                // writable even when the destination file itself is not there yet.
                try
                {
                    await StageAsync(SourcePath, Hash, Staged);
                }
                catch (Exception Exception) when (IsAccessDenied(Exception))
                {
                    await GrantAsync(CacheDirectory, Parent, true);

                    ValidatePath(Root, Destination);
                    ValidatePath(Root, Staged);

                    await StageAsync(SourcePath, Hash, Staged);
                }

                try
                {
                    CommitMove(Staged, Destination);
                }
                catch (Exception Exception) when (IsAccessDenied(Exception) || Exception is IOException)
                {
                    if (!IsAccessDenied(Exception))
                    {
                        throw;
                    }

                    if (File.Exists(Destination))
                    {
                        await GrantAsync(CacheDirectory, Destination, true);
                    }
                    else
                    {
                        await GrantAsync(CacheDirectory, Parent, true);
                    }

                    ValidatePath(Root, Destination);
                    ValidatePath(Root, Staged);

                    CommitMove(Staged, Destination);
                }

                string Actual = HashFile(Destination);

                if (!string.Equals(Hash, Actual, StringComparison.OrdinalIgnoreCase))
                {
                    throw new InvalidDataException("The committed destination hash did not match the source GIF.");
                }
            }
            finally
            {
                try
                {
                    File.Delete(Staged);
                }
                catch
                {
                }
            }
        }

        private static async Task StageAsync(string SourcePath, string Hash, string Staged)
        {
            using FileStream Source = new(SourcePath, FileMode.Open, FileAccess.Read, FileShare.Read);
            using FileStream Target = new(Staged, FileMode.CreateNew, FileAccess.ReadWrite, FileShare.None);

            await Source.CopyToAsync(Target);
            await Target.FlushAsync();

            Target.Position = 0;

            string StagedHash = Convert.ToHexString(await SHA256.HashDataAsync(Target));

            if (!string.Equals(Hash, StagedHash, StringComparison.OrdinalIgnoreCase))
            {
                throw new IOException("The staged file does not match the source; the previous destination was preserved.");
            }
        }

        private static void CommitMove(string Staged, string Destination)
        {
            FileAttributes? OriginalAttributes = null;
            bool Committed = false;

            try
            {
                if (File.Exists(Destination))
                {
                    FileAttributes Attributes = File.GetAttributes(Destination);

                    if ((Attributes & FileAttributes.ReadOnly) != 0)
                    {
                        File.SetAttributes(Destination, Attributes & ~FileAttributes.ReadOnly);
                        OriginalAttributes = Attributes;
                    }

                    // Replacing atomically preserves the existing ACL and metadata.
                    File.Replace(Staged, Destination, null);
                }
                else
                {
                    File.Move(Staged, Destination);
                }

                Committed = true;
            }
            finally
            {
                if (!Committed && OriginalAttributes.HasValue)
                {
                    try
                    {
                        File.SetAttributes(Destination, OriginalAttributes.Value);
                    }
                    catch
                    {
                    }
                }
            }
        }

        public static async Task<(int Succeeded, int Failed)> RemoveAppliedAsync(string CacheDirectory)
        {
            List<string> Destinations = FindDestinations(CacheDirectory);

            int Succeeded = 0;
            int Failed = 0;

            foreach (string Destination in Destinations)
            {
                try
                {
                    if (File.Exists(Destination))
                    {
                        File.Delete(Destination);
                    }

                    Succeeded++;
                }
                catch (Exception Exception) when (IsAccessDenied(Exception))
                {
                    try
                    {
                        await GrantAsync(CacheDirectory, Destination, true);

                        if (File.Exists(Destination))
                        {
                            File.Delete(Destination);
                        }

                        Succeeded++;
                    }
                    catch
                    {
                        Failed++;
                    }
                }
                catch
                {
                    Failed++;
                }
            }

            return (Succeeded, Failed);
        }

        private static string DescribeRecovery(Exception Exception)
        {
            if (Exception is not null && IsAccessDenied(Exception))
            {
                return "Windows denied access to the lock-screen cache. Retry and allow the permission request if it appears.";
            }

            if (Exception is DirectoryNotFoundException or FileNotFoundException)
            {
                return "The Windows lock-screen cache could not be found. Choose Picture in Windows lock-screen settings and retry.";
            }

            if (Exception is not null)
            {
                return $"The GIF could not be applied ({Exception.GetType().Name}: {Exception.Message}).";
            }

            return "The GIF could not be applied.";
        }

        public static string GetFFmpegPath()
        {
            return Path.Combine(
                Environment.GetFolderPath(Environment.SpecialFolder.ApplicationData),
                "Sucrose",
                "Cache",
                "FFmpeg",
                "ffmpeg.exe"
            );
        }

        public static async Task<string> EnsureFFmpegAsync(
            System.Net.Http.HttpClient Client,
            string Url,
            string Destination,
            IProgress<double> Progress,
            CancellationToken Cancellation
        )
        {
            if (File.Exists(Destination))
            {
                return Destination;
            }

            Directory.CreateDirectory(System.IO.Path.GetDirectoryName(Destination));

            string Archive = System.IO.Path.Combine(System.IO.Path.GetTempPath(), $"ffmpeg_{Guid.NewGuid():N}.zip");

            try
            {
                using System.Net.Http.HttpResponseMessage Response = await Client.GetAsync(
                    Url,
                    System.Net.Http.HttpCompletionOption.ResponseHeadersRead,
                    Cancellation);

                Response.EnsureSuccessStatusCode();

                long? Total = Response.Content.Headers.ContentLength;

                using Stream Remote = await Response.Content.ReadAsStreamAsync(Cancellation);
                using FileStream Local = new(Archive, FileMode.Create, FileAccess.Write, FileShare.None);

                byte[] Buffer = new byte[81920];
                long Received = 0;
                int Count;

                while ((Count = await Remote.ReadAsync(Buffer, Cancellation)) > 0)
                {
                    await Local.WriteAsync(Buffer.AsMemory(0, Count), Cancellation);
                    Received += Count;

                    if (Total > 0)
                    {
                        Progress?.Report(Received * 100.0 / Total.Value);
                    }
                }

                await Local.FlushAsync(Cancellation);
                Cancellation.ThrowIfCancellationRequested();

                // Extraction has no byte progress; the UI switches to indeterminate.
                Progress?.Report(-1);

                using System.IO.Compression.ZipArchive Entries = System.IO.Compression.ZipFile.OpenRead(Archive);

                System.IO.Compression.ZipArchiveEntry Binary = Entries.Entries.FirstOrDefault(
                    Entry => Entry.FullName.EndsWith("bin/ffmpeg.exe", StringComparison.OrdinalIgnoreCase)
                ) ?? throw new InvalidDataException("The FFmpeg package does not contain an executable.");

                Binary.ExtractToFile(Destination, true);

                return Destination;
            }
            finally
            {
                try
                {
                    File.Delete(Archive);
                }
                catch
                {
                }
            }
        }

        public static async Task<string> ConvertVideoToGifAsync(
            string FFmpeg,
            string Source,
            string OutputDirectory,
            int Width,
            double Fps,
            int MaxSeconds,
            bool Hardware,
            CancellationToken Cancellation,
            IProgress<string> Stage = null,
            IProgress<double> Progress = null,
            double TotalSeconds = 0
        )
        {
            try
            {
                return await ConvertCoreAsync(FFmpeg, Source, OutputDirectory, Width, Fps, MaxSeconds, Hardware, Cancellation, Stage, Progress, TotalSeconds);
            }
            catch (Exception) when (Hardware)
            {
                // Hardware paths depend on driver and codec support. A failed
                // attempt must not fail the whole job; clear its partial files
                // and retry once on CPU.
                Cancellation.ThrowIfCancellationRequested();

                Stage?.Report("CpuFallback");

                try
                {
                    foreach (string Partial in Directory.EnumerateFiles(OutputDirectory, "*.gif")
                        .Concat(Directory.EnumerateFiles(OutputDirectory, "palette_*.png")))
                    {
                        try
                        {
                            File.Delete(Partial);
                        }
                        catch
                        {
                        }
                    }
                }
                catch
                {
                }

                return await ConvertCoreAsync(FFmpeg, Source, OutputDirectory, Width, Fps, MaxSeconds, false, Cancellation, Stage, Progress, TotalSeconds);
            }
        }

        private static async Task<string> ConvertCoreAsync(
            string FFmpeg,
            string Source,
            string OutputDirectory,
            int Width,
            double Fps,
            int MaxSeconds,
            bool Hardware,
            CancellationToken Cancellation,
            IProgress<string> Stage,
            IProgress<double> Progress,
            double TotalSeconds
        )
        {
            Directory.CreateDirectory(OutputDirectory);

            string Output = System.IO.Path.Combine(OutputDirectory, $"lockscreen_{Guid.NewGuid():N}.gif");
            string Palette = System.IO.Path.Combine(OutputDirectory, $"palette_{Guid.NewGuid():N}.png");

            // Optional parts: width<=0 keeps source width, fps<=0 keeps all
            // source frames, seconds<=0 keeps the whole video. Hardware decode
            // needs an explicit download back to system memory for the CPU filters.
            string Prefix = Hardware ? "hwdownload,format=nv12," : string.Empty;
            string Input = Hardware ? "-hwaccel d3d11va " : string.Empty;
            string Filters = Prefix + string.Join(
                ",",
                new[] { Fps > 0 ? $"fps={Fps.ToString(System.Globalization.CultureInfo.InvariantCulture)}" : null, Width > 0 ? $"scale={Width}:-2:flags=lanczos" : null }.Where(
                    Part => Part is not null
                )
            );

            try
            {
                Cancellation.ThrowIfCancellationRequested();

                // Global palette over the whole clip, then per-frame palette
                // adaptation (new=1) with error diffusion for smoother gradients.
                Stage?.Report(Hardware ? "PaletteHw" : "Palette");

                List<string> PaletteLog = new();

                int PaletteExit = await RunFFmpegAsync(
                    FFmpeg,
                    $"{Input}{TimeLimit(MaxSeconds)}-i \"{Source}\" -vf \"{Filters},palettegen=max_colors=256\" \"{Palette}\"",
                    Cancellation,
                    Line =>
                    {
                        lock (PaletteLog)
                        {
                            PaletteLog.Add(Line);

                            if (PaletteLog.Count > 12)
                            {
                                PaletteLog.RemoveAt(0);
                            }
                        }

                        ReportTime(Line, Progress, TotalSeconds, 0);
                    });

                if (PaletteExit != 0 || !File.Exists(Palette))
                {
                    throw new InvalidOperationException("The video preview could not be analyzed." + DescribeLog(PaletteLog));
                }

                Stage?.Report(Hardware ? "GifHw" : "Gif");

                List<string> GifLog = new();

                int GifExit = await RunFFmpegAsync(
                    FFmpeg,
                    $"{Input}{TimeLimit(MaxSeconds)}-i \"{Source}\" -i \"{Palette}\" -lavfi \"{Filters} [x]; [x][1:v] paletteuse=dither=sierra2_4a:bayer_scale=5:diff_mode=rectangle:new=1\" \"{Output}\"",
                    Cancellation,
                    Line =>
                    {
                        lock (GifLog)
                        {
                            GifLog.Add(Line);

                            if (GifLog.Count > 12)
                            {
                                GifLog.RemoveAt(0);
                            }
                        }

                        ReportTime(Line, Progress, TotalSeconds, 50);
                    });

                if (GifExit != 0 || !File.Exists(Output) || !IsGif(Output))
                {
                    throw new InvalidOperationException("The video could not be converted to GIF." + DescribeLog(GifLog));
                }

                return Output;
            }
            finally
            {
                try
                {
                    File.Delete(Palette);
                }
                catch
                {
                }
            }
        }

        private static string DescribeLog(List<string> Log)
        {
            string Detail;

            lock (Log)
            {
                Detail = string.Join(" ", Log).Trim();
            }

            if (string.IsNullOrEmpty(Detail))
            {
                return string.Empty;
            }

            return Detail.Length > 300 ? " " + Detail[^300..] : " " + Detail;
        }

        private static void ReportTime(string Line, IProgress<double> Progress, double TotalSeconds, double Offset)
        {
            if (Progress is null || TotalSeconds <= 0)
            {
                return;
            }

            Match Time = Regex.Match(Line, @"time=(\d+):(\d+):([\d.]+)");

            if (!Time.Success)
            {
                return;
            }

            double Seconds =
                (int.Parse(Time.Groups[1].Value, System.Globalization.CultureInfo.InvariantCulture) * 3600)
                + (int.Parse(Time.Groups[2].Value, System.Globalization.CultureInfo.InvariantCulture) * 60)
                + double.Parse(Time.Groups[3].Value, System.Globalization.CultureInfo.InvariantCulture);

            Progress.Report(Math.Clamp(Offset + (Seconds / TotalSeconds * 50), 0, 100));
        }

        private static string TimeLimit(int MaxSeconds)
        {
            return MaxSeconds > 0 ? $"-t {MaxSeconds} " : string.Empty;
        }

        public sealed record VideoProbe(int Width, int Height, double Fps, double DurationSeconds);

        public static async Task<string> ExtractThumbnailAsync(
            string FFmpeg,
            string Source,
            string OutputDirectory,
            int Width,
            CancellationToken Cancellation = default
        )
        {
            Directory.CreateDirectory(OutputDirectory);

            string Output = System.IO.Path.Combine(OutputDirectory, "preview.png");

            int Exit = await RunFFmpegAsync(FFmpeg, $"-ss 1 -i \"{Source}\" -frames:v 1 -vf \"scale={Width}:-2\" \"{Output}\"", Cancellation);

            if (Exit != 0 || !File.Exists(Output))
            {
                throw new InvalidDataException("A video preview could not be extracted.");
            }

            return Output;
        }

        public static async Task<VideoProbe> ProbeVideoAsync(string FFmpeg, string Source, CancellationToken Cancellation = default)
        {
            Cancellation.ThrowIfCancellationRequested();

            ProcessStartInfo Start = new(FFmpeg, $"-hide_banner -nostdin -i \"{Source}\"")
            {
                UseShellExecute = false,
                CreateNoWindow = true,
                RedirectStandardOutput = true,
                RedirectStandardError = true
            };

            using Process Process = Process.Start(Start) ?? throw new IOException("FFmpeg did not start.");

            // Header-only probe: ffmpeg exits on its own, drain both pipes anyway.
            Task<string> Output = Process.StandardOutput.ReadToEndAsync();
            Task<string> Error = Process.StandardError.ReadToEndAsync();

            await Process.WaitForExitAsync(Cancellation);
            string Log = await Error;
            await Output;

            int Width = 0;
            int Height = 0;
            double Fps = 0;
            double Duration = 0;

            Match DurationMatch = System.Text.RegularExpressions.Regex.Match(Log, @"Duration:\s*(\d+):(\d+):([\d.]+)");
            if (DurationMatch.Success)
            {
                Duration =
                    (int.Parse(DurationMatch.Groups[1].Value, System.Globalization.CultureInfo.InvariantCulture) * 3600)
                    + (int.Parse(DurationMatch.Groups[2].Value, System.Globalization.CultureInfo.InvariantCulture) * 60)
                    + double.Parse(DurationMatch.Groups[3].Value, System.Globalization.CultureInfo.InvariantCulture);
            }

            Match VideoMatch = System.Text.RegularExpressions.Regex.Match(Log, @"Stream\s+#\d+:\d+.*Video:.*?(\d{2,5})x(\d{2,5})");
            if (VideoMatch.Success)
            {
                Width = int.Parse(VideoMatch.Groups[1].Value, System.Globalization.CultureInfo.InvariantCulture);
                Height = int.Parse(VideoMatch.Groups[2].Value, System.Globalization.CultureInfo.InvariantCulture);

                Match FpsMatch = System.Text.RegularExpressions.Regex.Match(VideoMatch.Value + Log[VideoMatch.Index..], @"([\d.]+)\s*fps");
                if (FpsMatch.Success)
                {
                    double.TryParse(FpsMatch.Groups[1].Value, System.Globalization.NumberStyles.Float, System.Globalization.CultureInfo.InvariantCulture, out Fps);
                }
            }

            if (Width <= 0 || Height <= 0)
            {
                throw new InvalidDataException("No video stream was found in the selected file.");
            }

            return new VideoProbe(Width, Height, Fps, Duration);
        }

        private static async Task<int> RunFFmpegAsync(
            string FFmpeg,
            string Arguments,
            CancellationToken Cancellation = default,
            Action<string> OnLine = null
        )
        {
            ProcessStartInfo Start = new(FFmpeg, "-nostdin " + Arguments)
            {
                UseShellExecute = false,
                CreateNoWindow = true,
                RedirectStandardOutput = true,
                RedirectStandardError = true
            };

            using Process Process = Process.Start(Start) ?? throw new IOException("FFmpeg did not start.");

            // Draining both pipes while waiting. FFmpeg logs progress to
            // stderr; an unread pipe fills up and blocks the process.
            // Cancelling kills the process so closing the UI aborts the work.
            using CancellationTokenRegistration Registration = Cancellation.Register(() =>
            {
                try
                {
                    if (!Process.HasExited)
                    {
                        Process.Kill();
                    }
                }
                catch (InvalidOperationException)
                {
                }
                catch (System.ComponentModel.Win32Exception)
                {
                }
            });

            Task Output = Process.StandardOutput.BaseStream.CopyToAsync(Stream.Null, Cancellation);

            try
            {
                string Line;

                while ((Line = await Process.StandardError.ReadLineAsync(Cancellation)) is not null)
                {
                    OnLine?.Invoke(Line);
                }

                await Output;
                await Process.WaitForExitAsync(Cancellation);
            }
            catch (OperationCanceledException)
            {
                try
                {
                    if (!Process.HasExited)
                    {
                        Process.Kill();
                    }
                }
                catch
                {
                }

                throw;
            }

            Cancellation.ThrowIfCancellationRequested();

            return Process.ExitCode;
        }

        [DllImport("user32.dll", CharSet = CharSet.Auto, SetLastError = true)]
        private static extern bool EnumDisplayDevices(string lpDevice, uint iDevNum, ref DISPLAY_DEVICE lpDisplayDevice, uint dwFlags);

        [DllImport("user32.dll", CharSet = CharSet.Ansi, SetLastError = true)]
        private static extern bool EnumDisplaySettings(string lpszDeviceName, int iModeNum, ref DEVMODE lpDevMode);

        [StructLayout(LayoutKind.Sequential, CharSet = CharSet.Auto)]
        private struct DISPLAY_DEVICE
        {
            public int cb;

            [MarshalAs(UnmanagedType.ByValTStr, SizeConst = 32)]
            public string DeviceName;

            [MarshalAs(UnmanagedType.ByValTStr, SizeConst = 128)]
            public string DeviceString;

            public int StateFlags;

            [MarshalAs(UnmanagedType.ByValTStr, SizeConst = 128)]
            public string DeviceID;

            [MarshalAs(UnmanagedType.ByValTStr, SizeConst = 128)]
            public string DeviceKey;
        }

        [StructLayout(LayoutKind.Sequential, CharSet = CharSet.Ansi)]
        private struct DEVMODE
        {
            [MarshalAs(UnmanagedType.ByValArray, SizeConst = 32)]
            public byte[] dmDeviceName;

            public short dmSpecVersion;
            public short dmDriverVersion;
            public short dmSize;
            public short dmDriverExtra;
            public int dmFields;
            public int dmPositionX;
            public int dmPositionY;
            public int dmDisplayOrientation;
            public int dmDisplayFixedOutput;
            public short dmColor;
            public short dmDuplex;
            public short dmYResolution;
            public short dmTTOption;
            public short dmCollate;

            [MarshalAs(UnmanagedType.ByValTStr, SizeConst = 32)]
            public string dmFormName;

            public short dmLogPixels;
            public int dmBitsPerPel;
            public int dmPelsWidth;
            public int dmPelsHeight;
            public int dmDisplayFlags;
            public int dmDisplayFrequency;
            public int dmICMMethod;
            public int dmICMIntent;
            public int dmMediaType;
            public int dmDitherType;
            public int dmReserved1;
            public int dmReserved2;
            public int dmPanningWidth;
            public int dmPanningHeight;
        }
    }
}
