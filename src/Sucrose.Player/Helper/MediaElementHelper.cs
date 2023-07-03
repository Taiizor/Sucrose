using Sucrose.Player.Manage;

namespace Sucrose.Player.Helper
{
    internal static class MediaElementHelper
    {
        public static void Pause()
        {
            Internal.MediaPlayer.Pause();
        }

        public static void Play()
        {
            Internal.MediaPlayer.Play();
        }

        public static void Stop()
        {
            Internal.MediaPlayer.Stop();
        }

        public static void SetVolume(int Volume)
        {
            Internal.MediaPlayer.Volume = (double)Volume / 100;
        }
    }
}