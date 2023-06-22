#if WPF_CS
    using Sucrose.Tray;
#endif
using Sucrose.Manager;

namespace Sucrose.Common.Manage
{
    internal static class Internal
    {
        public static SettingsManager ServerManager = new("Server.json");

#if WPF_CS
        public static TrayIconManager TrayIconManager = new();
#endif
    }
}