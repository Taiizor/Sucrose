using System.Diagnostics;
using WebEngine = Microsoft.Web.WebView2.Wpf.WebView2;

namespace Sucrose.Shared.Engine.WebView.Manage
{
    internal static class Internal
    {
        public static string Url = string.Empty;

        public static string Web = string.Empty;

        public static string Video = string.Empty;

        public static WebEngine WebEngine = new();

        public static string YouTube = string.Empty;

#if NET6_0_OR_GREATER
        public static int ProcessId = Environment.ProcessId;
#else
        public static int ProcessId = Process.GetCurrentProcess().Id;
#endif
    }
}