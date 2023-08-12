using System.Windows.Media.Imaging;
using SPSSS = Sucrose.Portal.Services.SearchService;

namespace Sucrose.Portal.Manage
{
    internal static class Internal
    {
        public static SPSSS SearchService { get; } = new();

        public static readonly Dictionary<string, BitmapImage> Images = new();
    }
}