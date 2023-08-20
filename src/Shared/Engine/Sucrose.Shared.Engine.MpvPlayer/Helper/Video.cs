using System.Windows.Media;
using SSEMPMI = Sucrose.Shared.Engine.MpvPlayer.Manage.Internal;

namespace Sucrose.Shared.Engine.MpvPlayer.Helper
{
    internal static class Video
    {
        public static void Pause()
        {
            SSEMPMI.MediaEngine.Pause();
        }

        public static void Play()
        {
            SSEMPMI.MediaEngine.Resume();
        }

        public static void Stop()
        {
            SSEMPMI.MediaEngine.Stop();
        }

        public static void SetLoop(bool State)
        {
            if (State && SSEMPMI.MediaEngine.Remaining <= TimeSpan.Zero)
            {
                SSEMPMI.MediaEngine.Load(SSEMPMI.Source, true);
            }

            SSEMPMI.MediaEngine.Loop = State;
        }

        public static void SetSpeed(double Speed)
        {
            SSEMPMI.MediaEngine.Speed = Speed;
        }

        public static void SetStretch(Stretch Mode)
        {
            KeyValuePair<string, string> Scaler = Mode switch
            {
                Stretch.None => new("video-unscaled", "yes"),
                Stretch.Fill => new("keepaspect", "no"),
                Stretch.Uniform => new("keepaspect", "yes"),
                Stretch.UniformToFill => new("panscan", "1.0"),
                _ => new("video-unscaled", "yes"),
            };

            SSEMPMI.MediaEngine.API.SetPropertyString(Scaler.Key, Scaler.Value);
        }

        public static void SetVolume(int Volume)
        {
            SSEMPMI.MediaEngine.Volume = Volume;
        }
    }
}