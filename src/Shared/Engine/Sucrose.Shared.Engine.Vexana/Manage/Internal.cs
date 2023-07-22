using System.Windows.Media;
using System.Windows.Threading;
using ImageEngine = System.Windows.Controls.Image;
using SSEVSG = Sucrose.Shared.Engine.Vexana.Struct.Gif;

namespace Sucrose.Shared.Engine.Vexana.Manage
{
    internal static class Internal
    {
        public static bool ImageLoop = true;

        public static bool ImageFirst = true;

        public static bool ImageState = true;

        public static SSEVSG ImageResult = new()
        {
            Min = 100,
            Max = 100,
            Total = 0,
            List = new()
        };

        public static ImageEngine ImageEngine = new()
        {
            Stretch = Stretch.Fill
        };

        public static readonly DispatcherTimer ImageTimer = new()
        {
            Interval = new TimeSpan(0, 0, 1),
        };
    }
}