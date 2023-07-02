using System.Globalization;
using SGMR = Sucrose.Globalization.Manage.Resources;

namespace Sucrose.Globalization.Helper
{
    public static class GeneralLocalization
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
            return SGMR.GeneralManager.GetString(Key, Culture);
        }
    }
}