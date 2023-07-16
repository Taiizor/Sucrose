using CefSharp;
using System.Windows;
using SECSMI = Sucrose.Engine.CS.Manage.Internal;
using SESHS = Sucrose.Engine.Shared.Helper.Source;
using SMC = Sucrose.Memory.Constant;
using SMMI = Sucrose.Manager.Manage.Internal;
using STSHV = Sucrose.Theme.Shared.Helper.Various;

namespace Sucrose.Engine.CS.Event
{
    internal static class YouTube
    {
        public static void CefEngineFrameLoadEnd(object sender, FrameLoadEndEventArgs e)
        {
            SECSMI.CefEngine.ExecuteScriptAsync($"setVolume({SMMI.EngineSettingManager.GetSettingStable(SMC.Volume, 100)});");
            SECSMI.CefEngine.ExecuteScriptAsync("toggleFullScreen();");
            SECSMI.CefEngine.ExecuteScriptAsync("playVideo();");
        }

        public static void CefEngineLoaded(object sender, RoutedEventArgs e)
        {
            string Video = STSHV.GetYouTubeVideoId(SECSMI.YouTube);
            string Playlist = STSHV.GetYouTubePlaylistId(SECSMI.YouTube);

            string Path = SESHS.GetYouTubeContentPath();

            SESHS.WriteYouTubeContent(Path, Video, Playlist);

            SECSMI.CefEngine.Address = SESHS.GetSource(Path).ToString();

            //SECSMI.CefEngine.ShowDevTools();
        }
    }
}