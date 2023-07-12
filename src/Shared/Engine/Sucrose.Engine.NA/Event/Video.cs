using System.Windows;
using SENAMI = Sucrose.Engine.NA.Manage.Internal;
using SMC = Sucrose.Memory.Constant;
using SMMI = Sucrose.Manager.Manage.Internal;

namespace Sucrose.Engine.NA.Event
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
                SENAMI.MediaEngine.Position = TimeSpan.Zero;
                SENAMI.MediaEngine.Play();
            }
        }
    }
}