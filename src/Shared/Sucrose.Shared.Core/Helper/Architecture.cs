namespace Sucrose.Shared.Core.Helper
{
    internal static class Architecture
    {
        public static string Get()
        {
#if X64
            return "x64";
#elif X86
            return "x86";
#elif ARM64
            return "ARM64";
#else
            return "Unknown";
#endif
        }
    }
}