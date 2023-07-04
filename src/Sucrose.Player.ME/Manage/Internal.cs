using System.Windows.Controls;
using System.Windows.Media;
using MediaPlayer = System.Windows.Controls.MediaElement;

namespace Sucrose.Player.ME.Manage
{
    internal static class Internal
    {
        public static MediaPlayer MediaPlayer = new()
        {
            LoadedBehavior = MediaState.Manual,
            Stretch = Stretch.Fill
        };
    }
}