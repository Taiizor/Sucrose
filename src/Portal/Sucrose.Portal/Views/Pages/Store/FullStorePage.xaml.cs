using Sucrose.Shared.Store.Interface;
using Sucrose.Shared.Theme.Helper;
using System.Collections.ObjectModel;
using System.IO;
using System.Windows;
using System.Windows.Controls;
using Wpf.Ui.Common;
using Wpf.Ui.Controls;
using SMC = Sucrose.Memory.Constant;
using SMMI = Sucrose.Manager.Manage.Internal;
using SMR = Sucrose.Memory.Readonly;
using SPMI = Sucrose.Portal.Manage.Internal;
using SPVCSC = Sucrose.Portal.Views.Controls.StoreCard;
using SSSHC = Sucrose.Shared.Space.Helper.Clean;
using SSSIC = Sucrose.Shared.Store.Interface.Category;
using SSSIR = Sucrose.Shared.Store.Interface.Root;
using SSSIW = Sucrose.Shared.Store.Interface.Wallpaper;
using SSTHI = Sucrose.Shared.Theme.Helper.Info;

namespace Sucrose.Portal.Views.Pages.Store
{
    /// <summary>
    /// FullStorePage.xaml etkileşim mantığı
    /// </summary>
    public partial class FullStorePage : Page, IDisposable
    {
        private static IList<char> Chars => Enumerable.Range('A', 'Z' - 'A' + 1).Concat(Enumerable.Range('a', 'z' - 'a' + 1)).Concat(Enumerable.Range('0', '9' - '0' + 1)).Select(C => (char)C).ToList();

        private static string LibraryLocation => SMMI.EngineSettingManager.GetSetting(SMC.LibraryLocation, Path.Combine(SMR.DocumentsPath, SMR.AppName));

        private static int AdaptiveLayout => SMMI.PortalSettingManager.GetSettingStable(SMC.AdaptiveLayout, 0);

        private static int AdaptiveMargin => SMMI.PortalSettingManager.GetSettingStable(SMC.AdaptiveMargin, 5);

        private static string Agent => SMMI.GeneralSettingManager.GetSetting(SMC.UserAgent, SMR.UserAgent);

        private static string Key => SMMI.PrivateSettingManager.GetSetting(SMC.Key, SMR.Key);

        private static bool Adult => SMMI.PortalSettingManager.GetSetting(SMC.Adult, false);

        private static Dictionary<string, SymbolRegular> MenuIcons { get; set; } = new()
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

        public static ICollection<object> MenuItems { get; set; }

        private SSSIR Root = new();

        internal FullStorePage(SSSIR Root)
        {
            this.Root = Root;
            DataContext = this;

            ObservableCollection<object> Categories = new();

            NavigationViewItem AllMenu = new("All Themes", SymbolRegular.Globe24, typeof(Canvas))
            {
                Tag = string.Empty,
                IsActive = SPMI.CategoryService.CategoryTag == string.Empty
            };

            AllMenu.Click += (s, e) => CategoryClick(s);

            Categories.Add(AllMenu);

            foreach (KeyValuePair<string, SSSIC> Category in Root.Categories)
            {
                if (Category.Value.Wallpapers.Any() && (Adult || Category.Value.Wallpapers.Count(Wallpaper => Wallpaper.Value.Adult) != Category.Value.Wallpapers.Count()))
                {
                    SymbolRegular Symbol = SymbolRegular.Wallpaper24;

                    if (MenuIcons.ContainsKey(Category.Key))
                    {
                        Symbol = MenuIcons[Category.Key];
                    }

                    NavigationViewItem Menu = new(Category.Key, Symbol, typeof(Border))
                    {
                        Tag = Category.Key,
                        IsActive = SPMI.CategoryService.CategoryTag == Category.Key
                    };

                    Menu.Click += (s, e) => CategoryClick(s);

                    Categories.Add(Menu);
                }
            }

            MenuItems = Categories;

            InitializeComponent();

            Category();
        }

        private void Category()
        {
            string Tag = SPMI.CategoryService.CategoryTag;

            SPMI.CategoryService.Dispose();

            SPMI.CategoryService = new()
            {
                CategoryTag = Tag
            };

            SPMI.CategoryService.CategoryTagChanged += CategoryService_CategoryTagChanged;
        }

        private void CategoryClick(object s)
        {
            NavigationViewItem sender = s as NavigationViewItem;

            SPMI.CategoryService.CategoryTag = sender.Tag.ToString();

            CategoryView
                .MenuItems
                .OfType<NavigationViewItem>()
                .Where(Item => Item.IsActive)
                .ToList()
                .ForEach(Item =>
                {
                    if (Item != sender)
                    {
                        Item.IsActive = false;
                    }
                });
        }

        private async Task AddThemes(string Search)
        {
            Dispose();

            foreach (KeyValuePair<string, SSSIC> Category in Root.Categories)
            {
                if (string.IsNullOrEmpty(SPMI.CategoryService.CategoryTag) || Category.Key == SPMI.CategoryService.CategoryTag)
                {
                    foreach (KeyValuePair<string, SSSIW> Wallpaper in Category.Value.Wallpapers)
                    {
                        if (!Wallpaper.Value.Adult || (Wallpaper.Value.Adult && Adult))
                        {
                            string Title = Wallpaper.Key.ToLowerInvariant();
                            string Theme = Path.Combine(SMR.AppDataPath, SMR.AppName, SMR.CacheFolder, SMR.Store, SSSHC.FileName(Wallpaper.Key));

                            if (SearchControl(Search, Theme, Title))
                            {
                                SPVCSC StoreCard = new(Theme, Wallpaper, Agent, Key);

                                ThemeStore.Children.Add(StoreCard);

                                Empty.Visibility = Visibility.Collapsed;

                                await Task.Delay(25);
                            }
                        }
                    }
                }
            }

            if (ThemeStore.Children.Count <= 0)
            {
                Empty.Visibility = Visibility.Visible;
            }
        }

        private bool SearchControl(string Search, string Theme, string Title)
        {
            if (string.IsNullOrEmpty(Search))
            {
                return true;
            }
            else
            {
                string InfoPath = Path.Combine(Theme, SMR.SucroseInfo);

                if (File.Exists(InfoPath))
                {
                    SSTHI Info = SSTHI.ReadJson(InfoPath);

                    Title = Info.Title.ToLowerInvariant();
                    string Description = Info.Description.ToLowerInvariant();

                    if (Title.Contains(Search) || Description.Contains(Search))
                    {
                        return true;
                    }
                }

                if (Title.Contains(Search))
                {
                    return true;
                }
            }

            return false;
        }

        private async void FullStorePage_Loaded(object sender, RoutedEventArgs e)
        {
            ThemeStore.ItemMargin = new Thickness(AdaptiveMargin);
            ThemeStore.MaxItemsPerRow = AdaptiveLayout;

            await AddThemes(SPMI.SearchService.SearchText);
        }

        private async void CategoryService_CategoryTagChanged(object sender, EventArgs e)
        {
            Dispose();

            await AddThemes(SPMI.SearchService.SearchText);
        }

        public void Dispose()
        {
            ThemeStore.Children.Clear();

            GC.Collect();
            GC.SuppressFinalize(this);
        }
    }
}