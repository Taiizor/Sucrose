using System.IO;
using SMMRF = Sucrose.Memory.Manage.Readonly.Folder;
using SMMRG = Sucrose.Memory.Manage.Readonly.General;
using SMMRP = Sucrose.Memory.Manage.Readonly.Path;
using SWML = Skylark.Wing.Manage.LockScreen;
#if RELEASE
using System.Runtime.InteropServices;
using SSSMI = Sucrose.Shared.Space.Manage.Internal;
#endif

namespace Sucrose.Shared.Dependency.Helper
{
    internal static class Runtime
    {
        public static void Configure()
        {
            SWML.RootDirectory = Path.Combine(SMMRP.CommonApplicationData, SMMRG.AppName, SMMRF.LockScreen);

#if RELEASE
            Environment.SetEnvironmentVariable("DOTNET_ROOT", SSSMI.Runtime, EnvironmentVariableTarget.Process);
            Environment.SetEnvironmentVariable("DOTNET_MULTILEVEL_LOOKUP", "0", EnvironmentVariableTarget.Process);
            Environment.SetEnvironmentVariable("DOTNET_ROLL_FORWARD", "LatestMajor", EnvironmentVariableTarget.Process);

            switch (RuntimeInformation.OSArchitecture)
            {
                case Architecture.X86:
                    Environment.SetEnvironmentVariable("DOTNET_ROOT(x86)", SSSMI.Runtime, EnvironmentVariableTarget.Process);
                    break;
                case Architecture.Arm64:
                    Environment.SetEnvironmentVariable("DOTNET_ROOT(arm64)", SSSMI.Runtime, EnvironmentVariableTarget.Process);
                    break;
                default:
                    break;
            }

            Environment.SetEnvironmentVariable("PATH", $"{SSSMI.Runtime};{SSSMI.This};{Environment.GetEnvironmentVariable("PATH") ?? string.Empty}", EnvironmentVariableTarget.Process);
#endif
        }
    }
}