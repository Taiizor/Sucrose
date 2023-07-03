using System.Windows.Controls;
using System.Windows.Media;
using MediaPlayer = System.Windows.Controls.MediaElement;
using EdgePlayer = Microsoft.Web.WebView2.Wpf.WebView2;

namespace Sucrose.Player.Manage
{
    internal static class Internal
    {
        public static MediaPlayer MediaPlayer = new()
        {
            LoadedBehavior = MediaState.Manual,
            Stretch = Stretch.Fill
        };

        public static EdgePlayer EdgePlayer = new()
        {
            //
        };
    }
}