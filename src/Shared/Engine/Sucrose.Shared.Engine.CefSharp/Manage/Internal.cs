using CefSharp;
using CefEngine = Sucrose.Shared.Engine.CefSharp.Control.CefBrowser;
using CefEngine2 = CefSharp.Wpf.ChromiumWebBrowser;

namespace Sucrose.Shared.Engine.CefSharp.Manage
{
    internal static class Internal
    {
        public static int Try = 0;

        public static bool State = true;

        public static string Gif = string.Empty;

        public static string Url = string.Empty;

        public static string Web = string.Empty;

        public static CefEngine CefEngine = new();

        public static List<int> Processes = new();

        public static CefEngine2 CefEngine2 = new();

        public static string YouTube = string.Empty;

        public static BrowserSettings CefSettings = new()
        {
            WindowlessFrameRate = 144
        };
    }
}