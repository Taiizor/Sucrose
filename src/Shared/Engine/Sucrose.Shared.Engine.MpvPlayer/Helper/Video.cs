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

        public static void SetVolume(int Volume)
        {
            SSEMPMI.MediaEngine.Volume = Volume;
        }

        public static void SetSpeed(double Speed)
        {
            SSEMPMI.MediaEngine.Speed = Speed;
        }

        public static void SetStretch(Stretch Mode)
        {
            string KeepAspect = Mode != Stretch.Fill ? "yes" : "no";
            string VideoUnscaled = Mode != Stretch.None ? "no" : "yes";
            string Panscan = Mode == Stretch.UniformToFill ? "1.0" : "0.0";

            SSEMPMI.MediaEngine.API.SetPropertyString("panscan", Panscan);
            SSEMPMI.MediaEngine.API.SetPropertyString("keepaspect", KeepAspect);
            SSEMPMI.MediaEngine.API.SetPropertyString("video-unscaled", VideoUnscaled);

            /*
                switch (Mode)
                {
                    case Stretch.None:
                        SSEMPMI.MediaEngine.API.SetPropertyString("panscan", "0.0");
                        SSEMPMI.MediaEngine.API.SetPropertyString("keepaspect", "yes");
                        SSEMPMI.MediaEngine.API.SetPropertyString("video-unscaled", "yes");
                        break;
                    case Stretch.Fill:
                        SSEMPMI.MediaEngine.API.SetPropertyString("panscan", "0.0");
                        SSEMPMI.MediaEngine.API.SetPropertyString("keepaspect", "no");
                        SSEMPMI.MediaEngine.API.SetPropertyString("video-unscaled", "no");
                        break;
                    case Stretch.Uniform:
                        SSEMPMI.MediaEngine.API.SetPropertyString("panscan", "0.0");
                        SSEMPMI.MediaEngine.API.SetPropertyString("keepaspect", "yes");
                        SSEMPMI.MediaEngine.API.SetPropertyString("video-unscaled", "no");
                        break;
                    case Stretch.UniformToFill:
                        SSEMPMI.MediaEngine.API.SetPropertyString("panscan", "1.0");
                        SSEMPMI.MediaEngine.API.SetPropertyString("keepaspect", "yes");
                        SSEMPMI.MediaEngine.API.SetPropertyString("video-unscaled", "no");
                        break;
                    default:
                        SSEMPMI.MediaEngine.API.SetPropertyString("panscan", "0.0");
                        SSEMPMI.MediaEngine.API.SetPropertyString("keepaspect", "no");
                        SSEMPMI.MediaEngine.API.SetPropertyString("video-unscaled", "no");
                        break;
                }
            */
        }
    }
}