using CefSharp;
using System.Windows;
using SESHD = Sucrose.Engine.Shared.Helper.Data;
using SESHS = Sucrose.Engine.Shared.Helper.Source;
using SESMI = Sucrose.Engine.Shared.Manage.Internal;
using SSECSMI = Sucrose.Shared.Engine.CefSharp.Manage.Internal;
using SSTHV = Sucrose.Shared.Theme.Helper.Various;

namespace Sucrose.Shared.Engine.CefSharp.Event
{
    internal static class YouTube
    {
        public static void CefEngineInitializedChanged(object sender, DependencyPropertyChangedEventArgs e)
        {
            SESMI.Initialized = SSECSMI.CefEngine.IsBrowserInitialized;
        }

        public static void CefEngineFrameLoadEnd(object sender, FrameLoadEndEventArgs e)
        {
            SSECSMI.CefEngine.ExecuteScriptAsync($"setVolume({SESHD.GetVolume()});");
            SSECSMI.CefEngine.ExecuteScriptAsync("toggleFullScreen();");
        }

        public static void CefEngineLoaded(object sender, RoutedEventArgs e)
        {
            string Video = SSTHV.GetYouTubeVideoId(SSECSMI.YouTube);
            string Playlist = SSTHV.GetYouTubePlaylistId(SSECSMI.YouTube);

            string Path = SESHS.GetYouTubeContentPath();

            SESHS.WriteYouTubeContent(Path, Video, Playlist);

            SSECSMI.CefEngine.Address = SESHS.GetSource(Path).ToString();

            //SSECSMI.CefEngine.ShowDevTools();
        }
    }
}