using System.Globalization;
using SGMR = Sucrose.Globalization.Manage.Resources;

namespace Sucrose.Globalization.Helper
{
    public static class MessageBoxLocalization
    {
        public static string GetValue(string Key)
        {
            return GetValue(Key, SGMR.CultureInfo);
        }

        public static string GetValue(string Key, string Culture)
        {
            return GetValue(Key, new CultureInfo(Culture, true));
        }

        public static string GetValue(string Key, CultureInfo Culture)
        {
            return SGMR.MessageBoxManager.GetString(Key, Culture);
        }
    }
}