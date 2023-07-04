using Microsoft.Web.WebView2.Core;
using Sucrose.Player.WV.Manage;

namespace Sucrose.Player.WV.Event
{
    internal static class Handler
    {
        public static void EdgePlayerInitializationCompleted(object sender, CoreWebView2InitializationCompletedEventArgs e)
        {
            Internal.EdgePlayer.CoreWebView2.DOMContentLoaded += EdgePlayerDOMContentLoaded;
        }

        public static void EdgePlayerDOMContentLoaded(object sender, CoreWebView2DOMContentLoadedEventArgs e)
        {
            Internal.EdgePlayer.CoreWebView2.ExecuteScriptAsync("document.getElementsByTagName('video')[0].requestFullscreen();");
            Internal.EdgePlayer.CoreWebView2.ExecuteScriptAsync("document.getElementsByTagName('video')[0].controls = false;");
            Internal.EdgePlayer.CoreWebView2.ExecuteScriptAsync("document.getElementsByTagName('video')[0].loop = true;");
        }
    }
}