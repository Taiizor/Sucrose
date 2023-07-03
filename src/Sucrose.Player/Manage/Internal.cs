using CefSharp;
using System.Windows.Controls;
using System.Windows.Media;
using CefPlayer = CefSharp.Wpf.ChromiumWebBrowser;
using EdgePlayer = Microsoft.Web.WebView2.Wpf.WebView2;
using MediaPlayer = System.Windows.Controls.MediaElement;

namespace Sucrose.Player.Manage
{
    internal static class Internal
    {
        public static MediaPlayer MediaPlayer = new()
        {
            LoadedBehavior = MediaState.Manual,
            Stretch = Stretch.Fill
        };

        public static EdgePlayer EdgePlayer = new()
        {
            //
        };

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