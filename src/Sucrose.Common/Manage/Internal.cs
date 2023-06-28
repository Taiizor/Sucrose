#if TRAY_ICON
using Sucrose.Tray.Manager;
#endif
#if SERVER || TRAY_ICON || GENERAL
using Sucrose.Manager;
#endif

namespace Sucrose.Common.Manage
{
    internal static class Internal
    {
        public static SettingManager GeneralSettingManager = new("General.json");

#if TRAY_ICON
        public static TrayIconManager TrayIconManager = new();
        public static LogManager TrayIconLogManager = new("TrayIcon-{0}.log");
        public static SettingManager TrayIconSettingManager = new("TrayIcon.json");
#endif

#if CEFSHARP
        public static LogManager CefSharpLogManager = new("CefSharp-{0}.log");
#endif

#if SERVER
        public static SettingManager ServerManager = new("Server.json");
#endif

#if BROWSER
        public static SettingManager WebsiteManager = new("Website.json");
#endif
    }
}