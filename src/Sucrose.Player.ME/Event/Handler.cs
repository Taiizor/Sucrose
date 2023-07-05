using System.Diagnostics;
using System.Windows;
using SMC = Sucrose.Memory.Constant;
using SMMI = Sucrose.Manager.Manage.Internal;
using SPMEMI = Sucrose.Player.ME.Manage.Internal;

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
            if (SMMI.EngineSettingManager.GetSetting(SMC.Loop, true))
            {
                SPMEMI.MediaPlayer.Position = TimeSpan.Zero;
                SPMEMI.MediaPlayer.Play();
            }
        }
    }
}