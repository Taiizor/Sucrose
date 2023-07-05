using System.Diagnostics;
using System.Windows;
using SPMEMI = Sucrose.Player.ME.Manage.Internal;
using SCMI = Sucrose.Common.Manage.Internal;
using SMC = Sucrose.Memory.Constant;

namespace Sucrose.Player.ME.Event
{
    internal static class Handler
    {
        public static void MediaPlayerOpened(object sender, RoutedEventArgs e)
        {
            Debug.WriteLine("Media playback opened: success");
        }

        public static void MediaPlayerFailed(object sender, ExceptionRoutedEventArgs e)
        {
            Debug.WriteLine("Media playback failure: " + e.ErrorException.Message);
        }

        public static void MediaPlayerEnded(object sender, EventArgs e)
        {
            if (SCMI.EngineSettingManager.GetSetting(SMC.Loop, true))
            {
                SPMEMI.MediaPlayer.Position = TimeSpan.Zero;
                SPMEMI.MediaPlayer.Play();
            }
        }
    }
}