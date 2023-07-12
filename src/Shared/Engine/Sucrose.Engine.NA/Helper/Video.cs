using SENAMI = Sucrose.Engine.NA.Manage.Internal;

namespace Sucrose.Engine.NA.Helper
{
    internal static class Video
    {
        public static void Pause()
        {
            SENAMI.MediaEngine.Pause();
        }

        public static void Play()
        {
            SENAMI.MediaEngine.Play();
        }

        public static void Stop()
        {
            SENAMI.MediaEngine.Stop();
        }

        public static void SetLoop(bool State)
        {
            if (State && (!SENAMI.MediaEngine.NaturalDuration.HasTimeSpan || SENAMI.MediaEngine.Position >= SENAMI.MediaEngine.NaturalDuration.TimeSpan))
            {
                SENAMI.MediaEngine.Position = TimeSpan.Zero;
                SENAMI.MediaEngine.Play();
            }
        }

        public static void SetVolume(int Volume)
        {
            SENAMI.MediaEngine.Volume = (double)Volume / 100;
        }
    }
}