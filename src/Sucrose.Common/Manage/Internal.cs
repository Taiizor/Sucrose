#if TRAY_ICON
using Sucrose.Tray;
#endif
using Sucrose.Manager;

namespace Sucrose.Common.Manage
{
    internal static class Internal
    {
        public static SettingsManager ServerManager = new("Server.json");

#if TRAY_ICON
        public static TrayIconManager TrayIconManager = new();
#endif
    }
}