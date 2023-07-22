using System.Windows.Controls;
using System.Windows.Media;
using MediaEngine = System.Windows.Controls.MediaElement;

namespace Sucrose.Shared.Engine.Nebula.Manage
{
    internal static class Internal
    {
        public static MediaEngine MediaEngine = new()
        {
            LoadedBehavior = MediaState.Manual,
            Stretch = Stretch.Fill,
            Volume = 0
        };
    }
}