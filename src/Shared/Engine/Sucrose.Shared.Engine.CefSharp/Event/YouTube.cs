using CefSharp;
using System.Windows;
using SSECSMI = Sucrose.Shared.Engine.CefSharp.Manage.Internal;
using SSEHD = Sucrose.Shared.Engine.Helper.Data;
using SSEHS = Sucrose.Shared.Engine.Helper.Source;
using SSEMI = Sucrose.Shared.Engine.Manage.Internal;
using SSTHV = Sucrose.Shared.Theme.Helper.Various;

namespace Sucrose.Shared.Engine.CefSharp.Event
{
    internal static class YouTube
    {
        public static void CefEngineInitializedChanged(object sender, DependencyPropertyChangedEventArgs e)
        {
            SSEMI.Initialized = SSECSMI.CefEngine.IsBrowserInitialized;
        }

        public static void CefEngineFrameLoadEnd(object sender, FrameLoadEndEventArgs e)
        {
            SSECSMI.CefEngine.ExecuteScriptAsync($"setVolume({SSEHD.GetVolume()});");
            SSECSMI.CefEngine.ExecuteScriptAsync("toggleFullScreen();");
        }

        public static void CefEngineLoaded(object sender, RoutedEventArgs e)
        {
            string Video = SSTHV.GetYouTubeVideoId(SSECSMI.YouTube);
            string Playlist = SSTHV.GetYouTubePlaylistId(SSECSMI.YouTube);

            string Path = SSEHS.GetYouTubeContentPath();

            SSEHS.WriteYouTubeContent(Path, Video, Playlist);

            SSECSMI.CefEngine.Address = SSEHS.GetSource(Path).ToString();

            //SSECSMI.CefEngine.ShowDevTools();
        }
    }
}