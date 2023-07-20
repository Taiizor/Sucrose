using Microsoft.Web.WebView2.Core;
using SESHD = Sucrose.Engine.Shared.Helper.Data;
using SESHS = Sucrose.Engine.Shared.Helper.Source;
using SEWVMI = Sucrose.Engine.WV.Manage.Internal;
using STSHV = Sucrose.Theme.Shared.Helper.Various;

namespace Sucrose.Engine.WV.Event
{
    internal static class YouTube
    {
        public static void WebEngineDOMContentLoaded(object sender, CoreWebView2DOMContentLoadedEventArgs e)
        {
            SEWVMI.WebEngine.CoreWebView2.ExecuteScriptAsync($"setVolume({SESHD.GetVolume()});");
            SEWVMI.WebEngine.CoreWebView2.ExecuteScriptAsync("toggleFullScreen();");
        }

        public static void WebEngineInitializationCompleted(object sender, CoreWebView2InitializationCompletedEventArgs e)
        {
            SEWVMI.Initialized = true;

            string Video = STSHV.GetYouTubeVideoId(SEWVMI.YouTube);
            string Playlist = STSHV.GetYouTubePlaylistId(SEWVMI.YouTube);

            string Path = SESHS.GetYouTubeContentPath();

            SESHS.WriteYouTubeContent(Path, Video, Playlist);

            SEWVMI.WebEngine.Source = SESHS.GetSource(Path);

            SEWVMI.WebEngine.CoreWebView2.DOMContentLoaded += WebEngineDOMContentLoaded;

            //SEWVMI.WebEngine.CoreWebView2.OpenDevToolsWindow();
        }
    }
}