#if TRAY_ICON
using STMTIM = Sucrose.Tray.Manager.TrayIconManager;
#endif
#if SERVER || TRAY_ICON || USER_INTERFACE || MEDIA_ELEMENT || GENERAL
using SMLM = Sucrose.Manager.LogManager;
using SMSM = Sucrose.Manager.SettingManager;
#endif

namespace Sucrose.Common.Manage
{
    internal static class Internal
    {
#if GENERAL
        public static SMSM GeneralSettingManager = new("General.json");
#endif

#if TRAY_ICON
        public static STMTIM TrayIconManager = new();
        public static SMLM TrayIconLogManager = new("TrayIcon-{0}.log");
        public static SMSM TrayIconSettingManager = new("TrayIcon.json");
#endif

#if USER_INTERFACE
        public static SMLM UserInterfaceLogManager = new("UserInterface-{0}.log");
#endif

#if MEDIA_ELEMENT
        public static SMLM MediaElementLogManager = new("MediaElement-{0}.log");
#endif

#if CEFSHARP
        public static SMLM CefSharpLogManager = new("CefSharp-{0}.log");
#endif

#if SERVER
        public static SMSM ServerManager = new("Server.json");
#endif

#if BROWSER
        public static SMSM WebsiteManager = new("Website.json");
#endif
    }
}