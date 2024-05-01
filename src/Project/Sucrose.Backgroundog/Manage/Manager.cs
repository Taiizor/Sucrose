namespace Sucrose.Backgroundog.Manage
{
    internal static class Manager
    {
        public static bool Windows11_OrGreater => Environment.OSVersion.Version.Build >= 22000;
    }
}