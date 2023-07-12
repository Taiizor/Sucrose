using System.Windows.Controls;
using System.Windows.Media;
using MediaEngine = System.Windows.Controls.MediaElement;

namespace Sucrose.Engine.NA.Manage
{
    internal static class Internal
    {
        public static MediaEngine MediaEngine = new()
        {
            LoadedBehavior = MediaState.Manual,
            Stretch = Stretch.Fill
        };
    }
}