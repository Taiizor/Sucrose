using SELLT = Skylark.Enum.LevelLogType;
using SMMI = Sucrose.Manager.Manage.Internal;

namespace Sucrose.Watchdog
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
#if CEFSHARP
            SMMI.CefSharpLogManager.Log(SELLT.Error, Text);
#elif TRAY_ICON
            SMMI.TrayIconLogManager.Log(SELLT.Error, Text);
#elif COMMANDOG
            SMMI.CommandogLogManager.Log(SELLT.Error, Text);
#elif USER_INTERFACE
            SMMI.UserInterfaceLogManager.Log(SELLT.Error, Text);
#elif WEBVIEW_PLAYER
            SMMI.WebViewPlayerLogManager.Log(SELLT.Error, Text);
#elif CEFSHARP_PLAYER
            SMMI.CefSharpPlayerLogManager.Log(SELLT.Error, Text);
#elif MEDIA_ELEMENT_PLAYER
            SMMI.MediaElementPlayerLogManager.Log(SELLT.Error, Text);
#endif
        }
    }
}