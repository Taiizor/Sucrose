using SEOST = Skylark.Enum.OperatingSystemType;
using SRISA = System.Runtime.InteropServices.Architecture;
using SWEOS = Skylark.Wing.Extension.OperatingSystem;
using SWHOS = Skylark.Wing.Helper.OperatingSystem;
using SWHSI = Skylark.Wing.Helper.SystemInfo;
using SWNM = Skylark.Wing.Native.Methods;

namespace Sucrose.Shared.Core.Helper
{
    internal static class OperatingSystem
    {
        public static SEOST Get()
        {
            return SWEOS.GetOperatingSystem();
        }

        public static string GetText()
        {
            return $"{Get()}";
        }

        public static bool GetServer()
        {
            return SWEOS.IsServer;
        }

        public static bool GetWorkstation()
        {
            return SWEOS.IsWorkstation;
        }

        public static int GetNumberOfProcessors()
        {
            SWNM.SYSTEM_INFO SystemInfo = SWHSI.GetSystemInfo();

            return Convert.ToInt32(SystemInfo.numberOfProcessors);
        }

        public static SRISA GetProcessArchitecture()
        {
            return SWHOS.GetProcessArchitecture();
        }

        public static string GetProcessorArchitecture()
        {
            return SWHOS.GetProcessorArchitecture();
        }

        public static string GetProcessArchitectureText()
        {
            return $"{GetProcessArchitecture()}";
        }
    }
}