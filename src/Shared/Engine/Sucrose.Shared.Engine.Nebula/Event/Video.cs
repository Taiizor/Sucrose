using System.Windows;
using SESHD = Sucrose.Engine.Shared.Helper.Data;
using SSENMI = Sucrose.Shared.Engine.Nebula.Manage.Internal;

namespace Sucrose.Shared.Engine.Nebula.Event
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
            if (SESHD.GetLoop())
            {
                SSENMI.MediaEngine.Position = TimeSpan.Zero;
                SSENMI.MediaEngine.Play();
            }
        }
    }
}