using System.Windows;
using XamlAnimatedGif;
using SMC = Sucrose.Memory.Constant;
using SMMI = Sucrose.Manager.Manage.Internal;
using SEVAMI = Sucrose.Engine.VA.Manage.Internal;
using SEVAHG = Sucrose.Engine.VA.Helper.Gif;

namespace Sucrose.Engine.VA.Event
{
    internal static class Gif
    {
        public static void AnimationLoadedEvent(object sender, RoutedEventArgs e)
        {
            SEVAMI.ImageAnimator = AnimationBehavior.GetAnimator(SEVAMI.ImageEngine);
        }

        public static void AnimationCompletedEvent(object sender, AnimationCompletedEventArgs e)
        {
            if (SMMI.EngineSettingManager.GetSetting(SMC.Loop, true) && SEVAMI.ImageAnimator.IsComplete)
            {
                SEVAHG.Play();
            }
        }
    }
}