using SMLM = Sucrose.Manager.LogManager;
using SMSM = Sucrose.Manager.SettingManager;

namespace Sucrose.Manager.Manage
{
    public static class Internal
    {
        public static readonly SMSM ServerManager = new("Server.json");

        public static readonly SMSM WebsiteManager = new("Website.json");

        public static readonly SMLM UpdateLogManager = new("Update-{0}.log");

        public static readonly SMLM PortalLogManager = new("Portal-{0}.log");

        public static readonly SMSM AuroraSettingManager = new("Aurora.json");

        public static readonly SMSM EngineSettingManager = new("Engine.json");

        public static readonly SMSM DiscordSettingManager = new("Discord.json");

        public static readonly SMSM GeneralSettingManager = new("General.json");

        public static readonly SMLM LauncherLogManager = new("Launcher-{0}.log");

        public static readonly SMSM LauncherSettingManager = new("Launcher.json");

        public static readonly SMLM CommandogLogManager = new("Commandog-{0}.log");

        public static readonly SMLM AuroraLiveLogManager = new("AuroraLive-{0}.log");

        public static readonly SMLM NebulaLiveLogManager = new("NebulaLive-{0}.log");

        public static readonly SMLM VexanaLiveLogManager = new("VexanaLive-{0}.log");

        public static readonly SMLM WebViewLiveLogManager = new("WebViewLive-{0}.log");

        public static readonly SMLM CefSharpLiveLogManager = new("CefSharpLive-{0}.log");
    }
}