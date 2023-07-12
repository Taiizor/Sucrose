using System.Windows;
using SEMEMI = Sucrose.Engine.ME.Manage.Internal;
using SMC = Sucrose.Memory.Constant;
using SMMI = Sucrose.Manager.Manage.Internal;

namespace Sucrose.Engine.ME.Event
{
    internal static class Video
    {
        public static void MediaEngineOpened(object sender, RoutedEventArgs e)
        {
            //Media playback opened: success
        }

        public static void MediaEngineFailed(object sender, ExceptionRoutedEventArgs e)
        {
            //Media playback failure: e.ErrorException.Message
        }

        public static void MediaEngineEnded(object sender, EventArgs e)
        {
            if (SMMI.EngineSettingManager.GetSetting(SMC.Loop, true))
            {
                SEMEMI.MediaEngine.Position = TimeSpan.Zero;
                SEMEMI.MediaEngine.Play();
            }
        }
    }
}