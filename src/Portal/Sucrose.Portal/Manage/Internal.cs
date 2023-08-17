using System.IO;
using System.Windows.Media.Imaging;
using Wpf.Ui.Contracts;
using SPSSS = Sucrose.Portal.Services.SearchService;

namespace Sucrose.Portal.Manage
{
    internal static class Internal
    {
        public static SPSSS SearchService = new();

        public static IServiceProvider ServiceProvider;

        public static ISnackbarService SnackbarService;

        public static INavigationService NavigationService;

        public static IContentDialogService ContentDialogService;

        public static readonly Dictionary<string, BitmapImage> Images = new();

        public static readonly Dictionary<string, FileStream> ImageStream = new();
    }
}