using System.Windows;

namespace Sucrose.XamlAnimatedGif
{
    public delegate void AnimationCompletedEventHandler(DependencyObject d, AnimationCompletedEventArgs e);

    public class AnimationCompletedEventArgs : RoutedEventArgs
    {
        public AnimationCompletedEventArgs(object source)
            : base(AnimationBehavior.AnimationCompletedEvent, source)
        {
        }
    }
}