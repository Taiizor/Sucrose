using CefSharp;
using CefPlayer = CefSharp.Wpf.ChromiumWebBrowser;

namespace Sucrose.Player.CS.Manage
{
    internal static class Internal
    {
        public static CefPlayer CefPlayer = new()
        {
            //BrowserSettings = CefSettings
        };

        public static BrowserSettings CefSettings = new()
        {
            WindowlessFrameRate = 144
        };
    }
}