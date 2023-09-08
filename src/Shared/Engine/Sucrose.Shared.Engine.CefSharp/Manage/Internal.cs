using CefSharp;
using System.Diagnostics;
using CefEngine = CefSharp.Wpf.ChromiumWebBrowser;

namespace Sucrose.Shared.Engine.CefSharp.Manage
{
    internal static class Internal
    {
        public static string Url = string.Empty;

        public static string Web = string.Empty;

        public static CefEngine CefEngine = new();

        public static string YouTube = string.Empty;

#if NET6_0_OR_GREATER
        public static int ProcessId = Environment.ProcessId;
#else
        public static int ProcessId = Process.GetCurrentProcess().Id;
#endif

        public static BrowserSettings CefSettings = new()
        {
            WindowlessFrameRate = 144
        };
    }
}