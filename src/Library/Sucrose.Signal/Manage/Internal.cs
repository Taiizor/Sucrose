using SSST = Sucrose.Signal.SignalT;

namespace Sucrose.Signal.Manage
{
    public static class Internal
    {
        public static readonly SSST PortalManager = new("Portal.sgnl");

        public static readonly SSST LauncherManager = new("Launcher.sgnl");

        public static readonly SSST WebsiterManager = new("Websiter.sgnl");
    }
}