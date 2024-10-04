using SSLMTIM = Sucrose.Shared.Launcher.Manager.TrayIconManager;

namespace Sucrose.Shared.Launcher.Manage
{
    internal static class Internal
    {
        public static bool ReportBox = true;

        public static bool FeedbackBox = false;

        public static SSLMTIM TrayIconManager = new();
    }
}