﻿using System.IO;
using System.Windows.Media.Imaging;
using Wpf.Ui;
using Wpf.Ui.Controls;
using SPSBS = Sucrose.Portal.Services.BackdropService;
using SPSCES = Sucrose.Portal.Services.CultureService;
using SPSCYS = Sucrose.Portal.Services.CategoryService;
using SPSDS = Sucrose.Portal.Services.DonateService;
using SPSLS = Sucrose.Portal.Services.LibraryService;
using SPSSS = Sucrose.Portal.Services.SearchService;

namespace Sucrose.Portal.Manage
{
    internal static class Internal
    {
        public static SPSDS DonateService = new();

        public static SPSSS SearchService = new();

        public static SPSLS LibraryService = new();

        public static SPSBS BackdropService = new();

        public static SPSCES CultureService = new();

        public static SPSCYS CategoryService = new();

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
            { "Brands", SymbolRegular.Tag24 },
            { "Game", SymbolRegular.Games24 },
            { "Time", SymbolRegular.Clock24 },
            { "Retro", SymbolRegular.Color24 },
            { "Sports", SymbolRegular.Sport24 },
            { "Fantasy", SymbolRegular.Crown20 },
            { "Line Art", SymbolRegular.Line24 },
            { "Galaxy", SymbolRegular.Rocket24 },
            { "Persons", SymbolRegular.Person24 },
            { "Anime", SymbolRegular.Filmstrip24 },
            { "Science", SymbolRegular.Syringe24 },
            { "Nature", SymbolRegular.EarthLeaf24 },
            { "Music", SymbolRegular.MusicNote224 },
            { "Abstract", SymbolRegular.HandDraw24 },
            { "Animals", SymbolRegular.AnimalCat24 },
            { "Cute", SymbolRegular.AnimalRabbit24 },
            { "Colorful", SymbolRegular.Lightbulb24 },
            { "Digital Art", SymbolRegular.Tablet24 },
            { "Medieval", SymbolRegular.Hourglass24 },
            { "Vehicles", SymbolRegular.VehicleCar24 },
            { "Lifestyle", SymbolRegular.StyleGuide24 },
            { "Comic Books", SymbolRegular.BookOpen24 },
            { "Food and Drinks", SymbolRegular.Food24 },
            { "Dynamic", SymbolRegular.ClockToolbox24 },
            { "System", SymbolRegular.ChartMultiple24 },
            { "Fashion", SymbolRegular.Accessibility24 },
            { "Animation", SymbolRegular.StarEmphasis24 },
            { "Cities and Places", SymbolRegular.City24 },
            { "Education", SymbolRegular.HatGraduation24 },
            { "Film and TV", SymbolRegular.MoviesAndTv24 },
            { "Technology", SymbolRegular.PhoneDesktop24 },
            { "Software", SymbolRegular.VirtualNetwork20 },
            { "Pixel Art", SymbolRegular.CalligraphyPen24 },
            { "Science Fiction", SymbolRegular.Thinking24 },
            { "Black and White", SymbolRegular.DarkTheme24 },
            { "Sky", SymbolRegular.WeatherPartlyCloudyDay24 },
            { "Geometric Patterns", SymbolRegular.Triangle20 },
            { "Underwater and Marine", SymbolRegular.Water24 },
            { "Ambience", SymbolRegular.HeadphonesSoundWave24 },
            { "Colorful Gradients", SymbolRegular.ColorFill24 },
            { "Landscape", SymbolRegular.RectangleLandscape24 },
            { "Clouds and Skies", SymbolRegular.WeatherCloudy24 },
            { "Enchanted Forests", SymbolRegular.TreeEvergreen20 },
            { "Minimalist", SymbolRegular.ArrowMinimizeVertical24 },
            { "Holidays and Seasons", SymbolRegular.SwimmingPool24 },
            { "Kids and Cartoon", SymbolRegular.VideoPersonSparkle48 }
        };
    }
}