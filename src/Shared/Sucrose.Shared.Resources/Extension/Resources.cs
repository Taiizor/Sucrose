using Application = System.Windows.Application;

namespace Sucrose.Shared.Resources.Extension
{
    internal static class Resources
    {
        public static string GetValue(string Key)
        {
            return GetResource(Key);
        }

        public static string GetValue(string Area, string Key)
        {
            return GetResource(Area + "." + Key);
        }

        public static string GetValue(string Area, string Prefix, string Key)
        {
            return GetResource(Area + "." + Prefix + "." + Key);
        }

        public static string GetValue(string Area, string Prefix, string Key, string Suffix)
        {
            return GetResource(Area + "." + Prefix + "." + Key + "." + Suffix);
        }

        private static string GetResource(string Resource)
        {
            string Result = Application.Current.TryFindResource(Resource) as string;

            if (string.IsNullOrEmpty(Result))
            {
                return $"[{Resource}]";
            }
            else
            {
                return Result;
            }
        }
    }
}