using System.Windows.Media;
using XamlAnimatedGif;
using ImageEngine = System.Windows.Controls.Image;

namespace Sucrose.Engine.VA.Manage
{
    internal static class Internal
    {
        public static ImageEngine ImageEngine = new()
        {
            Stretch = Stretch.Fill
        };

        public static Animator ImageAnimator;
    }
}