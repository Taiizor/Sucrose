using SEAT = Skylark.Enum.AssemblyType;
using SHV = Skylark.Helper.Versionly;
using SWHOS = Skylark.Wing.Helper.OperatingSystem;

namespace Sucrose.Shared.Core.Helper
{
    internal static class Version
    {
        public static string GetText()
        {
            return $"{Get()}";
        }

        public static string GetOSText()
        {
            return $"{GetOS()}";
        }

        public static System.Version Get()
        {
            return SHV.Auto(SEAT.Entry);
        }

        public static System.Version GetOS()
        {
            return SWHOS.GetVersion();
        }
    }
}