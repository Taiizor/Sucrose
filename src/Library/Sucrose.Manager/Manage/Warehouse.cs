using SMMCW = Sucrose.Memory.Manage.Constant.Warehouse;
using SMMI = Sucrose.Manager.Manage.Internal;

namespace Sucrose.Manager.Manage
{
    public static class Warehouse
    {
        public static List<string> Showcase => SMMI.WarehouseSettingManager.GetSetting(SMMCW.Showcase, new List<string>());

        public static DateTime CefSharpTime => SMMI.WarehouseSettingManager.GetSetting(SMMCW.CefSharpTime, new DateTime());

        public static DateTime WebViewTime => SMMI.WarehouseSettingManager.GetSetting(SMMCW.WebViewTime, new DateTime());

        public static bool CefsharpContinue => SMMI.WarehouseSettingManager.GetSetting(SMMCW.CefsharpContinue, false);

        public static bool WebViewContinue => SMMI.WarehouseSettingManager.GetSetting(SMMCW.WebViewContinue, false);

        public static bool HintTrayIcon => SMMI.WarehouseSettingManager.GetSetting(SMMCW.HintTrayIcon, true);
    }
}