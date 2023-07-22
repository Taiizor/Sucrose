using SELLT = Skylark.Enum.LevelLogType;
using SMMI = Sucrose.Manager.Manage.Internal;

namespace Sucrose.Shared.Watchdog
{
    internal static class Watch
    {
        public static void Watch_CatchException(Exception Exception)
        {
            WatchLog(Exception, "CATCH");
        }

        public static void Watch_ThreadException(Exception Exception)
        {
            WatchLog(Exception, "THREAD");
        }

        public static void Watch_FirstChanceException(Exception Exception)
        {
            WatchLog(Exception, "FIRST CHANCE");
        }

        public static void Watch_UnobservedTaskException(Exception Exception)
        {
            WatchLog(Exception, "UNOBSERVED TASK");
        }

        public static void Watch_DispatcherUnhandledException(Exception Exception)
        {
            WatchLog(Exception, "DISPATCHER UNHANDLED");
        }

        public static void Watch_GlobalUnhandledExceptionHandler(Exception Exception)
        {
            WatchLog(Exception, "GLOBAL UNHANDLED");
        }

        private static void WatchLog(Exception Exception, string Type)
        {
            WriteLog($"{Type} EXCEPTION START");
            WriteLog($"Application crashed: {Exception.Message}.");
            WriteLog($"Inner exception: {Exception.InnerException}.");
            WriteLog($"Stack trace: {Exception.StackTrace}.");
            WriteLog($"{Type} EXCEPTION FINISH");
        }

        private static void WriteLog(string Text)
        {
#if PORTAL
            SMMI.PortalLogManager.Log(SELLT.Error, Text);
#elif LAUNCHER
            SMMI.LauncherLogManager.Log(SELLT.Error, Text);
#elif COMMANDOG
            SMMI.CommandogLogManager.Log(SELLT.Error, Text);
#elif LIVE_AURORA
            SMMI.AuroraLiveLogManager.Log(SELLT.Error, Text);
#elif LIVE_NEBULA
            SMMI.NebulaLiveLogManager.Log(SELLT.Error, Text);
#elif LIVE_VEXANA
            SMMI.VexanaLiveLogManager.Log(SELLT.Error, Text);
#elif LIVE_WEBVIEW
            SMMI.WebViewLiveLogManager.Log(SELLT.Error, Text);
#elif LIVE_CEFSHARP
            SMMI.CefSharpLiveLogManager.Log(SELLT.Error, Text);
#endif
        }
    }
}