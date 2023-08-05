namespace Sucrose.Shared.Core.Helper
{
    internal static class Framework
    {
        public static string Get()
        {
#if NET48
            return ".NET Framework 4.8";
#elif NET481
            return ".NET Framework 4.8.1";
#elif NET6_0
            return ".NET 6.0";
#elif NET7_0
            return ".NET 7.0";
#elif NET8_0
            return ".NET 8.0";
#else
            return "Unknown";
#endif
        }
    }
}