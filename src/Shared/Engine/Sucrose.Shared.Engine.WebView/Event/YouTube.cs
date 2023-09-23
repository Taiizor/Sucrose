using Microsoft.Web.WebView2.Core;
using SMMM = Sucrose.Manager.Manage.Manager;
using SSEHD = Sucrose.Shared.Engine.Helper.Data;
using SSEHS = Sucrose.Shared.Engine.Helper.Source;
using SSEMI = Sucrose.Shared.Engine.Manage.Internal;
using SSEWVMI = Sucrose.Shared.Engine.WebView.Manage.Internal;
using SSTHV = Sucrose.Shared.Theme.Helper.Various;

namespace Sucrose.Shared.Engine.WebView.Event
{
    internal static class YouTube
    {
        public static void WebEngineDOMContentLoaded(object sender, CoreWebView2DOMContentLoadedEventArgs e)
        {
            SSEWVMI.WebEngine.CoreWebView2.ExecuteScriptAsync($"setVolume({SSEHD.GetVolume()});");
            SSEWVMI.WebEngine.CoreWebView2.ExecuteScriptAsync("toggleFullScreen();");
        }

        public static void WebEngineInitializationCompleted(object sender, CoreWebView2InitializationCompletedEventArgs e)
        {
            SSEMI.Initialized = e.IsSuccess;

            SSEWVMI.WebEngine.CoreWebView2.Settings.UserAgent = SMMM.UserAgent;

            string Video = SSTHV.GetYouTubeVideoId(SSEWVMI.YouTube);
            string Playlist = SSTHV.GetYouTubePlaylistId(SSEWVMI.YouTube);

            string Path = SSEHS.GetYouTubeContentPath();

            SSEHS.WriteYouTubeContent(Path, Video, Playlist);

            SSEWVMI.WebEngine.Source = SSEHS.GetSource(Path);

            SSEWVMI.WebEngine.CoreWebView2.DOMContentLoaded += WebEngineDOMContentLoaded;

            if (SMMM.DeveloperMode)
            {
                SSEWVMI.WebEngine.CoreWebView2.OpenDevToolsWindow();
            }
        }
    }
}