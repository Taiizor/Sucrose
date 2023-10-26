using System.Windows;

namespace Sucrose.Resources.Extension
{
    public static class Resources
    {
        public static string GetValue(string Key)
        {
            return GetResource(Key, GetBack(Key));
        }

        public static string GetValue(string Area, string Key)
        {
            return GetResource(Area + "." + Key, GetBack(Area, Key));
        }

        public static string GetValue(string Area, string Prefix, string Key)
        {
            return GetResource(Area + "." + Prefix + "." + Key, GetBack(Area, Prefix, Key));
        }

        public static string GetValue(string Area, string Prefix, string Key, string Suffix)
        {
            return GetResource(Area + "." + Prefix + "." + Key + "." + Suffix, GetBack(Area, Prefix, Key, Suffix));
        }

        public static string GetValue(string Area, string Prefix, string Key, string Suffix, string Last)
        {
            return GetResource(Area + "." + Prefix + "." + Key + "." + Suffix + "." + Last, GetBack(Area, Prefix, Key, Suffix, Last));
        }

        public static T GetResource<T>(string Resource, T Back = default)
        {
            object Result = Application.Current.TryFindResource(Resource);

            if (Result is not null and T)
            {
                return (T)Result;
            }
            else
            {
                return Back;
            }
        }

        private static string GetBack(string Key)
        {
            return $"[{Key}]";
        }

        private static string GetBack(string Area, string Key)
        {
            return GetBack(Area + "." + Key);
        }

        private static string GetBack(string Area, string Prefix, string Key)
        {
            return GetBack(Area + "." + Prefix + "." + Key);
        }

        private static string GetBack(string Area, string Prefix, string Key, string Suffix)
        {
            return GetBack(Area + "." + Prefix + "." + Key + "." + Suffix);
        }

        private static string GetBack(string Area, string Prefix, string Key, string Suffix, string Last)
        {
            return GetBack(Area + "." + Prefix + "." + Key + "." + Suffix + "." + Last);
        }
    }
}