using SRISA = System.Runtime.InteropServices.Architecture;
using SEOST = Skylark.Enum.OperatingSystemType;
using SWHOS = Skylark.Wing.Helper.OperatingSystem;

namespace Sucrose.Shared.Core.Helper
{
    internal static class OperatingSystem
    {
        public static SEOST Get()
        {
            return SWHOS.GetSystem();
        }

        public static string GetText()
        {
            return $"{Get()}";
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