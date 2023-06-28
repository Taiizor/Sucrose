using Sucrose.Globalization.Manage;
using System.Globalization;

namespace Sucrose.Globalization.Helper
{
    public static class MessageBoxLocalization
    {
        public static string GetValue(string Key)
        {
            return GetValue(Key, Resources.CultureInfo);
        }

        public static string GetValue(string Key, string Culture)
        {
            return GetValue(Key, new CultureInfo(Culture, true));
        }

        public static string GetValue(string Key, CultureInfo Culture)
        {
            return Resources.MessageBoxManager.GetString(Key, Culture);
        }
    }
}