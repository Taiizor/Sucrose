using CefSharp;
using CefEngine = CefSharp.Wpf.ChromiumWebBrowser;

namespace Sucrose.Engine.CS.Manage
{
    internal static class Internal
    {
        public static CefEngine CefEngine = new()
        {
            //BrowserSettings = CefSettings
        };

        public static BrowserSettings CefSettings = new()
        {
            WindowlessFrameRate = 144
        };
    }
}