using SELLT = Skylark.Enum.LevelLogType;
using SMMI = Sucrose.Manager.Manage.Internal;

namespace Sucrose.Shared.Watchdog
{
    internal static class Watch
    {
        public static async Task Watch_CatchException(Exception Exception)
        {
            await WatchLog(Exception, "CATCH");
        }

        public static async Task Watch_ThreadException(Exception Exception)
        {
            await WatchLog(Exception, "THREAD");
        }

        public static async Task Watch_FirstChanceException(Exception Exception)
        {
            await WatchLog(Exception, "FIRST CHANCE");
        }

        public static async Task Watch_UnobservedTaskException(Exception Exception)
        {
            await WatchLog(Exception, "UNOBSERVED TASK");
        }

        public static async Task Watch_GlobalUnhandledException(Exception Exception)
        {
            await WatchLog(Exception, "GLOBAL UNHANDLED");
        }

        public static async Task Watch_DispatcherUnhandledException(Exception Exception)
        {
            await WatchLog(Exception, "DISPATCHER UNHANDLED");
        }

        private static async Task WatchLog(Exception Exception, string Type)
        {
            await WriteLog($"{Type} EXCEPTION START");
            await WriteLog($"Application crashed: {Exception.Message}.");
            await WriteLog($"Inner exception: {Exception.InnerException}.");
            await WriteLog($"Stack trace: {Exception.StackTrace}.");
            await WriteLog($"{Type} EXCEPTION FINISH");
        }

        private static Task WriteLog(string Text)
        {
#if PORTAL
            SMMI.PortalLogManager.Log(SELLT.Error, Text);
#elif UPDATE
            SMMI.UpdateLogManager.Log(SELLT.Error, Text);
#elif LAUNCHER
            SMMI.LauncherLogManager.Log(SELLT.Error, Text);
#elif PROPERTY
            SMMI.PropertyLogManager.Log(SELLT.Error, Text);
#elif WATCHDOG
            SMMI.WatchdogLogManager.Log(SELLT.Error, Text);
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
#elif BACKGROUNDOG
            SMMI.BackgroundogLogManager.Log(SELLT.Error, Text);
#elif LIVE_CEFSHARP
            SMMI.CefSharpLiveLogManager.Log(SELLT.Error, Text);
#elif LIVE_MPVPLAYER
            SMMI.MpvPlayerLiveLogManager.Log(SELLT.Error, Text);
#endif

            return Task.CompletedTask;
        }
    }
}