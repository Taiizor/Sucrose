using System.Globalization;
using SGHL = Sucrose.Globalization.Helper.Localization;
using SGMR = Sucrose.Globalization.Manage.Resources;

namespace Sucrose.Globalization.Helper
{
    public static class WatchdogLocalization
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
            return SGMR.WatchdogManager.GetString(Key, Culture) ?? SGHL.FlexKey(Key);
        }
    }
}