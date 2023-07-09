using SEMEMI = Sucrose.Engine.ME.Manage.Internal;

namespace Sucrose.Engine.ME.Helper
{
    internal static class Video
    {
        public static void Pause()
        {
            SEMEMI.MediaEngine.Pause();
        }

        public static void Play()
        {
            SEMEMI.MediaEngine.Play();
        }

        public static void Stop()
        {
            SEMEMI.MediaEngine.Stop();
        }

        public static void SetLoop(bool State)
        {
            if (State && (!SEMEMI.MediaEngine.NaturalDuration.HasTimeSpan || SEMEMI.MediaEngine.Position >= SEMEMI.MediaEngine.NaturalDuration.TimeSpan))
            {
                SEMEMI.MediaEngine.Position = TimeSpan.Zero;
                SEMEMI.MediaEngine.Play();
            }
        }

        public static void SetVolume(int Volume)
        {
            SEMEMI.MediaEngine.Volume = (double)Volume / 100;
        }
    }
}