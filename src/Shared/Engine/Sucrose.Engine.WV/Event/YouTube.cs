using Microsoft.Web.WebView2.Core;
using SESHS = Sucrose.Engine.Shared.Helper.Source;
using SEWVMI = Sucrose.Engine.WV.Manage.Internal;
using SMC = Sucrose.Memory.Constant;
using SMMI = Sucrose.Manager.Manage.Internal;
using STSHV = Sucrose.Theme.Shared.Helper.Various;

namespace Sucrose.Engine.WV.Event
{
    internal static class YouTube
    {
        public static void WebEngineDOMContentLoaded(object sender, CoreWebView2DOMContentLoadedEventArgs e)
        {
            SEWVMI.WebEngine.CoreWebView2.ExecuteScriptAsync($"setVolume({SMMI.EngineSettingManager.GetSettingStable(SMC.Volume, 100)});");
            SEWVMI.WebEngine.CoreWebView2.ExecuteScriptAsync("toggleFullScreen();");
            SEWVMI.WebEngine.CoreWebView2.ExecuteScriptAsync("playVideo();");
        }

        public static void WebEngineInitializationCompleted(object sender, CoreWebView2InitializationCompletedEventArgs e)
        {
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