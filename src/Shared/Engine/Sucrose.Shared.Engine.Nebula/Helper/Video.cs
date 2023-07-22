using SSENMI = Sucrose.Shared.Engine.Nebula.Manage.Internal;

namespace Sucrose.Shared.Engine.Nebula.Helper
{
    internal static class Video
    {
        public static void Pause()
        {
            SSENMI.MediaEngine.Pause();
        }

        public static void Play()
        {
            SSENMI.MediaEngine.Play();
        }

        public static void Stop()
        {
            SSENMI.MediaEngine.Stop();
        }

        public static void SetLoop(bool State)
        {
            if (State && (!SSENMI.MediaEngine.NaturalDuration.HasTimeSpan || SSENMI.MediaEngine.Position >= SSENMI.MediaEngine.NaturalDuration.TimeSpan))
            {
                SSENMI.MediaEngine.Position = TimeSpan.Zero;
                SSENMI.MediaEngine.Play();
            }
        }

        public static void SetVolume(int Volume)
        {
            SSENMI.MediaEngine.Volume = (double)Volume / 100;
        }
    }
}