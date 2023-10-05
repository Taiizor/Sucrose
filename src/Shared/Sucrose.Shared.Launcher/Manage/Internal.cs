using SMR = Sucrose.Memory.Readonly;
using SSLMTIM = Sucrose.Shared.Launcher.Manager.TrayIconManager;

namespace Sucrose.Shared.Launcher.Manage
{
    internal static class Internal
    {
        public static bool ReportBox = true;

        public static SSLMTIM TrayIconManager = new();

        public static Mutex Mutex = new(true, SMR.LauncherMutex);
    }
}