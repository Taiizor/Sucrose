using Microsoft.Web.WebView2.Core;
using SMMM = Sucrose.Manager.Manage.Manager;
using SSEHS = Sucrose.Shared.Engine.Helper.Source;
using SSEMI = Sucrose.Shared.Engine.Manage.Internal;
using SSEWVHYT = Sucrose.Shared.Engine.WebView.Helper.YouTube;
using SSEWVMI = Sucrose.Shared.Engine.WebView.Manage.Internal;
using SSTHV = Sucrose.Shared.Theme.Helper.Various;

namespace Sucrose.Shared.Engine.WebView.Event
{
    internal static class YouTube
    {
        public static void WebEngineDOMContentLoaded(object sender, CoreWebView2DOMContentLoadedEventArgs e)
        {
            SSEWVHYT.Load();

            SSEMI.Initialized = true;
        }

        public static void WebEngineInitializationCompleted(object sender, CoreWebView2InitializationCompletedEventArgs e)
        {
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