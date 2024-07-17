using System.Windows.Media.Animation;
using SSEMI = Sucrose.Shared.Engine.Manage.Internal;
using SSEXMI = Sucrose.Shared.Engine.Xavier.Manage.Internal;
using SSWW = Sucrose.Shared.Watchdog.Watch;
using SXAGAB = Sucrose.XamlAnimatedGif.AnimationBehavior;

namespace Sucrose.Shared.Engine.Xavier.Helper
{
    internal static class Gif
    {
        public static void Play()
        {
            SXAGAB.GetAnimator(SSEXMI.ImageEngine).Play();
        }

        public static void Pause()
        {
            SXAGAB.GetAnimator(SSEXMI.ImageEngine).Pause();
        }

        public static void Resume()
        {
            SXAGAB.GetAnimator(SSEXMI.ImageEngine).Resume();
        }

        public static void Rewind()
        {
            SXAGAB.GetAnimator(SSEXMI.ImageEngine).Rewind();
        }

        public static void SetLoop(bool State)
        {
            if (State)
            {
                if (SXAGAB.GetRepeatBehavior(SSEXMI.ImageEngine) != RepeatBehavior.Forever && !SSEMI.PausePerformance)
                {
                    SXAGAB.SetRepeatBehavior(SSEXMI.ImageEngine, RepeatBehavior.Forever);

                    Uri Source = SXAGAB.GetSourceUri(SSEXMI.ImageEngine);

                    SSEXMI.ImageEngine.Source = null;
                    SXAGAB.SetSourceUri(SSEXMI.ImageEngine, null);

                    SXAGAB.SetSourceUri(SSEXMI.ImageEngine, Source);
                }
            }
            else
            {
                SXAGAB.SetRepeatBehavior(SSEXMI.ImageEngine, new RepeatBehavior(1));
            }
        }

        public static async void SetMemory(bool State)
        {
            try
            {
                if (SXAGAB.GetCacheFramesInMemory(SSEXMI.ImageEngine) != State)
                {
                    SXAGAB.SetCacheFramesInMemory(SSEXMI.ImageEngine, State);
                }
            }
            catch (Exception Exception)
            {
                await SSWW.Watch_CatchException(Exception);
            }
        }
    }
}