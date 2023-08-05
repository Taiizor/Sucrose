using SEAT = Skylark.Enum.AssemblyType;
using SHV = Skylark.Helper.Versionly;

namespace Sucrose.Shared.Core.Helper
{
    internal static class Version
    {
        public static string GetText()
        {
            return $"{SHV.Auto(SEAT.Entry)}";
        }

        public static System.Version Get()
        {
            return SHV.Auto(SEAT.Entry);
        }
    }
}