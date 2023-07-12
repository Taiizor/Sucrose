using SMLM = Sucrose.Manager.LogManager;
using SMSM = Sucrose.Manager.SettingManager;

namespace Sucrose.Manager.Manage
{
    public static class Internal
    {
        public static readonly SMSM ServerManager = new("Server.json");

        public static readonly SMSM WebsiteManager = new("Website.json");

        public static readonly SMSM EngineSettingManager = new("Engine.json");

        public static readonly SMSM DiscordSettingManager = new("Discord.json");

        public static readonly SMSM GeneralSettingManager = new("General.json");

        public static readonly SMLM TrayIconLogManager = new("TrayIcon-{0}.log");

        public static readonly SMSM TrayIconSettingManager = new("TrayIcon.json");

        public static readonly SMLM CommandogLogManager = new("Commandog-{0}.log");

        public static readonly SMLM AuroraLiveLogManager = new("AuroraLive-{0}.log");

        public static readonly SMLM NebulaLiveLogManager = new("NebulaLive-{0}.log");

        public static readonly SMLM VexanaLiveLogManager = new("VexanaLive-{0}.log");

        public static readonly SMLM WebViewLiveLogManager = new("WebViewLive-{0}.log");

        public static readonly SMLM CefSharpLiveLogManager = new("CefSharpLive-{0}.log");

        public static readonly SMLM UserInterfaceLogManager = new("UserInterface-{0}.log");
    }
}