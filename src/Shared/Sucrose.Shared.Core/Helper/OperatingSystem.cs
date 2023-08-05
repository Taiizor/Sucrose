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
    }
}