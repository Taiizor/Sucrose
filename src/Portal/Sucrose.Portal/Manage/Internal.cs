using System.IO;
using System.Windows.Media.Imaging;
using Wpf.Ui;
using Wpf.Ui.Controls;
using SPSCS = Sucrose.Portal.Services.CategoryService;
using SPSSS = Sucrose.Portal.Services.SearchService;

namespace Sucrose.Portal.Manage
{
    internal static class Internal
    {
        public static List<string> Themes = new();

        public static SPSSS SearchService = new();

        public static SPSCS CategoryService = new();

        public static IServiceProvider ServiceProvider;

        public static ISnackbarService SnackbarService;

        public static INavigationService NavigationService;

        public static IContentDialogService ContentDialogService;
        public static Dictionary<string, bool> StoreDownloader = new();

        public static Dictionary<string, bool> StoreDownloading = new();

        public static readonly SymbolRegular AllIcon = SymbolRegular.Home24;

        public static readonly Dictionary<string, BitmapImage> Images = new();

        public static readonly Dictionary<string, FileStream> ImageStream = new();

        public static readonly SymbolRegular DefaultIcon = SymbolRegular.Wallpaper24;

        public static readonly Dictionary<string, SymbolRegular> CategoryIcons = new()
        {
            { "Game", SymbolRegular.Games24 },
            { "RGB", SymbolRegular.Lightbulb24 },
            { "Music", SymbolRegular.MusicNote224 },
            { "Sky", SymbolRegular.WeatherCloudy24 },
            { "Animals", SymbolRegular.AnimalCat24 },
            { "Vehicles", SymbolRegular.VehicleCar24 },
            { "Dynamic", SymbolRegular.ClockToolbox24 },
            { "Film and Movie", SymbolRegular.MoviesAndTv24 }
        };
    }
}