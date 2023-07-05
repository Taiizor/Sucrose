using SPMEMI = Sucrose.Player.ME.Manage.Internal;

namespace Sucrose.Player.ME.Helper
{
    internal static class MediaElementHelper
    {
        public static void Pause()
        {
            SPMEMI.MediaPlayer.Pause();
        }

        public static void Play()
        {
            SPMEMI.MediaPlayer.Play();
        }

        public static void Stop()
        {
            SPMEMI.MediaPlayer.Stop();
        }

        public static void SetLoop(bool State)
        {
            if (State && (!SPMEMI.MediaPlayer.NaturalDuration.HasTimeSpan || SPMEMI.MediaPlayer.Position >= SPMEMI.MediaPlayer.NaturalDuration.TimeSpan))
            {
                SPMEMI.MediaPlayer.Position = TimeSpan.Zero;
                SPMEMI.MediaPlayer.Play();
            }
        }

        public static void SetVolume(int Volume)
        {
            SPMEMI.MediaPlayer.Volume = (double)Volume / 100;
        }
    }
}