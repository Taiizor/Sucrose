using SSDEAT = Sucrose.Shared.Dependency.Enum.ArchitectureType;

namespace Sucrose.Shared.Core.Helper
{
    internal static class Architecture
    {
        public static SSDEAT Get()
        {
#if X64
            return SSDEAT.x64;
#elif X86
            return SSDEAT.x86;
#elif ARM64
            return SSDEAT.ARM64;
#else
            return return SSDEAT.Unknown";
#endif
        }

        public static string GetText()
        {
            return $"{Get()}";
        }
    }
}