using System.IO;
using System.Windows;
using System.Windows.Controls;
using SMC = Sucrose.Memory.Constant;
using SMMI = Sucrose.Manager.Manage.Internal;
using SPMI = Sucrose.Portal.Manage.Internal;
using SPVCSC = Sucrose.Portal.Views.Controls.StoreCard;
using SPCSWP = Sucrose.Portal.Controls.StoreWrapPanel;
using SSTHI = Sucrose.Shared.Theme.Helper.Info;
using SMR = Sucrose.Memory.Readonly;
using SPVMPSVM = Sucrose.Portal.ViewModels.Pages.StoreViewModel;
using SPVPSFSP = Sucrose.Portal.Views.Pages.Store.FullStorePage;
using SPVPSBSP = Sucrose.Portal.Views.Pages.Store.BrokenStorePage;
using SSSHN = Sucrose.Shared.Space.Helper.Network;
using SSSHC = Sucrose.Shared.Space.Helper.Clean;
using SSSIR = Sucrose.Shared.Store.Interface.Root;
using SSSIC = Sucrose.Shared.Store.Interface.Category;
using SSSIW = Sucrose.Shared.Store.Interface.Wallpaper;
using System.Collections.ObjectModel;
using Wpf.Ui.Common;
using Wpf.Ui.Controls;

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

        private static Dictionary<string, SymbolRegular> MenuIcons { get; set; } = new();

        public static ICollection<object> MenuItems { get; set; }

        private SSSIR Root = new();

        internal FullStorePage(SSSIR Root)
        {
            this.Root = Root;
            DataContext = this;

            ObservableCollection<object> Categories = new();

            foreach (KeyValuePair<string, SSSIC> Category in Root.Categories)
            {
                if (Category.Value.Wallpapers.Any() && (Adult || Category.Value.Wallpapers.Count(Wallpaper => Wallpaper.Value.Adult) != Category.Value.Wallpapers.Count()))
                {
                    NavigationViewItem Menu = new(Category.Key, SymbolRegular.Wallpaper24, typeof(Border))
                    {
                        Tag = Category.Key
                    };

                    Menu.Click += (s, e) =>
                    {
                        NavigationViewItem sender = s as NavigationViewItem;

                        //System.Windows.MessageBox.Show(sender.Tag.ToString());
                    };

                    Categories.Add(Menu);
                }
            }

            MenuItems = Categories;

            InitializeComponent();
        }

        private async Task AddThemes(string Search)
        {
            Dispose();

            foreach (KeyValuePair<string, SSSIC> Category in Root.Categories)
            {
                //MessageBox.Show("Category: " + Category.Key);

                foreach (KeyValuePair<string, SSSIW> Wallpaper in Category.Value.Wallpapers)
                {
                    //MessageBox.Show("Wallpaper: " + Wallpaper.Key);

                    //MessageBox.Show("Source: " + Wallpaper.Value.Source);
                    //MessageBox.Show("Adult: " + Wallpaper.Value.Adult);
                    //MessageBox.Show("Cover: " + Wallpaper.Value.Cover);
                    //MessageBox.Show("Live: " + Wallpaper.Value.Live);

                    if (!Wallpaper.Value.Adult || (Wallpaper.Value.Adult && Adult))
                    {
                        string Theme = Path.Combine(SMR.AppDataPath, SMR.AppName, SMR.CacheFolder, SMR.Store, SSSHC.FileName(Wallpaper.Key));

                        SPVCSC StoreCard = new(Theme, Wallpaper, Agent, Key);

                        ThemeStore.Children.Add(StoreCard);

                        Empty.Visibility = Visibility.Collapsed;

                        await Task.Delay(25);
                    }
                }
            }

            if (ThemeStore.Children.Count <= 0)
            {
                Empty.Visibility = Visibility.Visible;
            }
        }

        private async void FullStorePage_Loaded(object sender, RoutedEventArgs e)
        {
            ThemeStore.ItemMargin = new Thickness(AdaptiveMargin);
            ThemeStore.MaxItemsPerRow = AdaptiveLayout;

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