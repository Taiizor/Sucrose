using Microsoft.Web.WebView2.Core;
using SESHD = Sucrose.Engine.Shared.Helper.Data;
using SESHS = Sucrose.Engine.Shared.Helper.Source;
using SESMI = Sucrose.Engine.Shared.Manage.Internal;
using SSEWVMI = Sucrose.Shared.Engine.WebView.Manage.Internal;
using SSTHV = Sucrose.Shared.Theme.Helper.Various;

namespace Sucrose.Shared.Engine.WebView.Event
{
    internal static class YouTube
    {
        public static void WebEngineDOMContentLoaded(object sender, CoreWebView2DOMContentLoadedEventArgs e)
        {
            SSEWVMI.WebEngine.CoreWebView2.ExecuteScriptAsync($"setVolume({SESHD.GetVolume()});");
            SSEWVMI.WebEngine.CoreWebView2.ExecuteScriptAsync("toggleFullScreen();");
        }

        public static void WebEngineInitializationCompleted(object sender, CoreWebView2InitializationCompletedEventArgs e)
        {
            SESMI.Initialized = e.IsSuccess;

            string Video = SSTHV.GetYouTubeVideoId(SSEWVMI.YouTube);
            string Playlist = SSTHV.GetYouTubePlaylistId(SSEWVMI.YouTube);

            string Path = SESHS.GetYouTubeContentPath();

            SESHS.WriteYouTubeContent(Path, Video, Playlist);

            SSEWVMI.WebEngine.Source = SESHS.GetSource(Path);

            SSEWVMI.WebEngine.CoreWebView2.DOMContentLoaded += WebEngineDOMContentLoaded;

            //SSEWVMI.WebEngine.CoreWebView2.OpenDevToolsWindow();
        }
    }
}