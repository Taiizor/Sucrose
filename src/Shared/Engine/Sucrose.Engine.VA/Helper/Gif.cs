using System.Windows.Media.Animation;
using XamlAnimatedGif;
using SEVAMI = Sucrose.Engine.VA.Manage.Internal;

namespace Sucrose.Engine.VA.Helper
{
    internal static class Gif
    {
        public static void Pause()
        {
            SEVAMI.ImageAnimator.Pause();
        }

        public static void Play()
        {
            SEVAMI.ImageAnimator.Play();
        }

        public static void SetLoop(bool State)
        {
            if (State)
            {
                AnimationBehavior.SetRepeatBehavior(SEVAMI.ImageEngine, RepeatBehavior.Forever);

                Play();
            }
            else
            {
                AnimationBehavior.SetRepeatBehavior(SEVAMI.ImageEngine, new RepeatBehavior(1));
            }
        }
    }
}