using System.Windows.Controls;
using System.Windows.Media;
using ImageEngine = System.Windows.Controls.Image;

namespace Sucrose.Shared.Engine.Xavier.Manage
{
    internal static class Internal
    {
        public static ImageEngine ImageEngine = new()
        {
            Stretch = Stretch.Fill,
            StretchDirection = StretchDirection.Both
        };
    }
}