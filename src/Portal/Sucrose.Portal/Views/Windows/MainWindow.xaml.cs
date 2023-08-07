using Sucrose.Portal.Services.Contracts;
using Sucrose.Portal.ViewModels;
using System.IO;
using System.Windows;
using SEWTT = Skylark.Enum.WindowsThemeType;
using SHG = Skylark.Helper.Generator;
using SMC = Sucrose.Memory.Constant;
using SMMI = Sucrose.Manager.Manage.Internal;
using SMR = Sucrose.Memory.Readonly;
using SSSHD = Sucrose.Shared.Store.Helper.Download;
using SWHWT = Skylark.Wing.Helper.WindowsTheme;
using WUAT = Wpf.Ui.Appearance.ApplicationThemeManager;
using WUAAT = Wpf.Ui.Appearance.ApplicationTheme;
using SSSHS = Sucrose.Shared.Store.Helper.Store;
using SSSIR = Sucrose.Shared.Store.Interface.Root;
using SSSIC = Sucrose.Shared.Store.Interface.Category;
using SSSIW = Sucrose.Shared.Store.Interface.Wallpaper;
using SPVPLP = Sucrose.Portal.Views.Pages.LibraryPage;
using SPVPLFLP = Sucrose.Portal.Views.Pages.Library.FullLibraryPage;
using SPVPLELP = Sucrose.Portal.Views.Pages.Library.EmptyLibraryPage;

namespace Sucrose.Portal.Views.Windows
{
    /// <summary>
    /// Interaction logic for MainWindow.xaml
    /// </summary>
    public partial class MainWindow : IWindow
    {
        private static IList<char> Chars => Enumerable.Range('A', 'Z' - 'A' + 1).Concat(Enumerable.Range('a', 'z' - 'a' + 1)).Concat(Enumerable.Range('0', '9' - '0' + 1)).Select(C => (char)C).ToList();

        private static string Directory => SMMI.EngineSettingManager.GetSetting(SMC.Directory, Path.Combine(SMR.DocumentsPath, SMR.AppName));

        private static SEWTT Theme => SMMI.GeneralSettingManager.GetSetting(SMC.ThemeType, SWHWT.GetTheme());

        private static string Agent => SMMI.GeneralSettingManager.GetSetting(SMC.UserAgent, SMR.UserAgent);

        private static string Key => SMMI.PrivateSettingManager.GetSetting(SMC.Key, SMR.Key);

        public MainWindowViewModel ViewModel { get; }

        public MainWindow(MainWindowViewModel viewModel)
        {
            ViewModel = viewModel;
            DataContext = this;

            if (Theme == SEWTT.Dark)
            {
                WUAT.Apply(WUAAT.Dark);
            }
            else
            {
                WUAT.Apply(WUAAT.Light);
            }

            InitializeComponent();

            View.Loaded += (_, _) => View.Navigate(typeof(SPVPLP));

            //string StoreFile = Path.Combine(SMR.AppDataPath, SMR.AppName, SMR.CacheFolder, SMR.Store, SMR.StoreFile);

            //if (SSSHD.Store(StoreFile, Agent, Key))
            //{
            //    //MessageBox.Show(SSSHS.Json(StoreFile));

            //    SSSIR Root = SSSHS.DeserializeRoot(StoreFile);

            //    foreach (KeyValuePair<string, SSSIC> Category in Root.Categories)
            //    {
            //        //MessageBox.Show("Kategori: " + Category.Key);

            //        foreach (KeyValuePair<string, SSSIW> Wallpaper in Category.Value.Wallpapers)
            //        {
            //            //MessageBox.Show("Duvar Kağıdı: " + Wallpaper.Key);

            //            //MessageBox.Show("Kaynak: " + Wallpaper.Value.Source);
            //            //MessageBox.Show("Kapak: " + Wallpaper.Value.Cover);
            //            //MessageBox.Show("Canlı: " + Wallpaper.Value.Live);

            //            string Keys = SHG.GenerateString(Chars, 25, SMR.Randomise);
            //            bool Result = SSSHD.Theme(Path.Combine(Wallpaper.Value.Source, Wallpaper.Key), Path.Combine(Directory, Keys), Agent, Keys, Key).Result;
            //        }
            //    }
            //}
        }

        private void MainWindow_SizeChanged(object sender, SizeChangedEventArgs e)
        {
            double WindowWidth = e.NewSize.Width;
            double SearchWidth = SearchBox.RenderSize.Width;

            SearchBox.Margin = new Thickness(0, 0, ((WindowWidth - SearchWidth) / 2) - 165, 0);
        }
    }
}