#if TRAY_ICON
using Sucrose.Tray.Manager;
#endif
#if SERVER || TRAY_ICON
using Sucrose.Manager;
#endif

namespace Sucrose.Common.Manage
{
    internal static class Internal
    {
#if TRAY_ICON
        public static TrayIconManager TrayIcon = new();
        public static SettingsManager TrayIconManager = new("TrayIcon.json");
#endif

#if SERVER
        public static SettingsManager ServerManager = new("Server.json");
#endif

#if BROWSER
        public static SettingsManager WebsiteManager = new("Website.json");
#endif
    }
}