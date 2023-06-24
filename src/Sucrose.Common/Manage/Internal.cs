#if TRAY_ICON
using Sucrose.Tray;
#endif
#if SERVER
using Sucrose.Manager;
#endif

namespace Sucrose.Common.Manage
{
    internal static class Internal
    {
#if TRAY_ICON
        public static TrayIconManager TrayIconManager = new();
#endif

#if SERVER
        public static SettingsManager ServerManager = new("Server.json");
#endif

#if BROWSER
        public static SettingsManager WebsiteManager = new("Website.json");
#endif
    }
}