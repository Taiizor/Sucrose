using CefSharp;
using System.Windows;
using SECSMI = Sucrose.Engine.CS.Manage.Internal;
using SESHD = Sucrose.Engine.Shared.Helper.Data;
using SESHS = Sucrose.Engine.Shared.Helper.Source;
using SESMI = Sucrose.Engine.Shared.Manage.Internal;
using SSTHV = Sucrose.Shared.Theme.Helper.Various;

namespace Sucrose.Engine.CS.Event
{
    internal static class YouTube
    {
        public static void CefEngineInitializedChanged(object sender, DependencyPropertyChangedEventArgs e)
        {
            SESMI.Initialized = SECSMI.CefEngine.IsBrowserInitialized;
        }

        public static void CefEngineFrameLoadEnd(object sender, FrameLoadEndEventArgs e)
        {
            SECSMI.CefEngine.ExecuteScriptAsync($"setVolume({SESHD.GetVolume()});");
            SECSMI.CefEngine.ExecuteScriptAsync("toggleFullScreen();");
        }

        public static void CefEngineLoaded(object sender, RoutedEventArgs e)
        {
            string Video = SSTHV.GetYouTubeVideoId(SECSMI.YouTube);
            string Playlist = SSTHV.GetYouTubePlaylistId(SECSMI.YouTube);

            string Path = SESHS.GetYouTubeContentPath();

            SESHS.WriteYouTubeContent(Path, Video, Playlist);

            SECSMI.CefEngine.Address = SESHS.GetSource(Path).ToString();

            //SECSMI.CefEngine.ShowDevTools();
        }
    }
}