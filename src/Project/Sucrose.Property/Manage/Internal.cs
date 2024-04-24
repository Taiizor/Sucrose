using SSTHP = Sucrose.Shared.Theme.Helper.Properties;

namespace Sucrose.Property.Manage
{
    internal static class Internal
    {
        public static string Path;

        public static string InfoPath;

        public static string PropertiesFile;

        public static string PropertiesPath;

        public static string PropertiesCache;

        public static bool EngineLive = false;

        public static SSTHP Properties = new();
    }
}