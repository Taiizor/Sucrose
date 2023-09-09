using CefSharp;
using CefEngine = CefSharp.Wpf.ChromiumWebBrowser;

namespace Sucrose.Shared.Engine.CefSharp.Manage
{
    internal static class Internal
    {
        public static int Try = 0;

        public static string Url = string.Empty;

        public static string Web = string.Empty;

        public static CefEngine CefEngine = new();

        public static List<int> Processes = new();

        public static string YouTube = string.Empty;

        public static BrowserSettings CefSettings = new()
        {
            WindowlessFrameRate = 144
        };
    }
}