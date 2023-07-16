using CefSharp;
using CefEngine = CefSharp.Wpf.ChromiumWebBrowser;

namespace Sucrose.Engine.CS.Manage
{
    internal static class Internal
    {
        public static CefEngine CefEngine = new();

        public static string YouTube = string.Empty;

        public static BrowserSettings CefSettings = new()
        {
            WindowlessFrameRate = 144
        };
    }
}