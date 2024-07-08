using CefSharp;
using System.Windows;
using SMMM = Sucrose.Manager.Manage.Manager;
using SSECSHYT = Sucrose.Shared.Engine.CefSharp.Helper.YouTube;
using SSECSMI = Sucrose.Shared.Engine.CefSharp.Manage.Internal;
using SSEHS = Sucrose.Shared.Engine.Helper.Source;
using SSEMI = Sucrose.Shared.Engine.Manage.Internal;
using SSTHV = Sucrose.Shared.Theme.Helper.Various;

namespace Sucrose.Shared.Engine.CefSharp.Event
{
    internal static class YouTube
    {
        public static void CefEngineLoaded(object sender, RoutedEventArgs e)
        {
            string Video = SSTHV.GetYouTubeVideoId(SSECSMI.YouTube);
            string Playlist = SSTHV.GetYouTubePlaylistId(SSECSMI.YouTube);

            string Path = SSEHS.GetYouTubeContentPath();

            SSEHS.WriteYouTubeContent(Path, Video, Playlist);

            SSECSMI.CefEngine.Address = SSEHS.GetSource(Path).ToString();
        }

        public static void CefEngineInitializedChanged(object sender, EventArgs e)
        {
            if (SMMM.DeveloperMode)
            {
                SSECSMI.CefEngine.ShowDevTools();
            }

            SSEMI.Initialized = SSECSMI.CefEngine.IsBrowserInitialized;
        }

        public static void CefEngineFrameLoadEnd(object sender, FrameLoadEndEventArgs e)
        {
            SSECSHYT.Load();
        }
    }
}