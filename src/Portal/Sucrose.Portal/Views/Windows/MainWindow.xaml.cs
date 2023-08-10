﻿using Sucrose.Portal.Services.Contracts;
using Sucrose.Portal.ViewModels.Windows;
using System.IO;
using System.Windows;
using System.Windows.Controls;
using Wpf.Ui.Contracts;
using Wpf.Ui.Controls;
using Button = Wpf.Ui.Controls.Button;
using SEWTT = Skylark.Enum.WindowsThemeType;
using SMC = Sucrose.Memory.Constant;
using SMMI = Sucrose.Manager.Manage.Internal;
using SMR = Sucrose.Memory.Readonly;
using SPVPLP = Sucrose.Portal.Views.Pages.LibraryPage;
using SPVPSGSP = Sucrose.Portal.Views.Pages.Setting.GeneralSettingPage;
using SSDEACT = Sucrose.Shared.Dependency.Enum.ArgumentCommandsType;
using SWHWT = Skylark.Wing.Helper.WindowsTheme;
using WUAAT = Wpf.Ui.Appearance.ApplicationTheme;
using WUAT = Wpf.Ui.Appearance.ApplicationThemeManager;

namespace Sucrose.Portal.Views.Windows
{
    /// <summary>
    /// Interaction logic for MainWindow.xaml
    /// </summary>
    public partial class MainWindow : IWindow, IDisposable
    {
        private static IList<char> Chars => Enumerable.Range('A', 'Z' - 'A' + 1).Concat(Enumerable.Range('a', 'z' - 'a' + 1)).Concat(Enumerable.Range('0', '9' - '0' + 1)).Select(C => (char)C).ToList();

        private static string Directory => SMMI.EngineSettingManager.GetSetting(SMC.Directory, Path.Combine(SMR.DocumentsPath, SMR.AppName));

        private static SEWTT Theme => SMMI.GeneralSettingManager.GetSetting(SMC.ThemeType, SWHWT.GetTheme());

        private static string Agent => SMMI.GeneralSettingManager.GetSetting(SMC.UserAgent, SMR.UserAgent);

        private static string Key => SMMI.PrivateSettingManager.GetSetting(SMC.Key, SMR.Key);

        public MainWindowViewModel ViewModel { get; }

        public MainWindow(MainWindowViewModel ViewModel, INavigationService NavigationService, IServiceProvider ServiceProvider, ISnackbarService SnackbarService, IContentDialogService ContentDialogService)
        {
            this.ViewModel = ViewModel;
            DataContext = this;

            InitializeComponent();

            if (Theme == SEWTT.Dark)
            {
                WUAT.Apply(WUAAT.Dark);
                Light.Visibility = Visibility.Collapsed;
            }
            else
            {
                WUAT.Apply(WUAAT.Light);
                Dark.Visibility = Visibility.Collapsed;
            }

            RootView.SetServiceProvider(ServiceProvider);

            NavigationService.SetNavigationControl(RootView);
            SnackbarService.SetSnackbarPresenter(SnackbarPresenter);
            ContentDialogService.SetContentPresenter(RootContentDialog);

            string[] Args = Environment.GetCommandLineArgs();

            if (Args.Count() > 1 && Args[1] == $"{SSDEACT.Setting}")
            {
                ApplySetting(false);
                RootView.Loaded += (_, _) => RootView.Navigate(typeof(SPVPSGSP));
            }
            else
            {
                ApplyGeneral(false);
                RootView.Loaded += (_, _) => RootView.Navigate(typeof(SPVPLP));
            }

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

        private void ApplyTheme(Button Button)
        {
            if (Button.Name == "Dark")
            {
                Dark.Visibility = Visibility.Collapsed;
                Light.Visibility = Visibility.Visible;
            }
            else
            {
                Dark.Visibility = Visibility.Visible;
                Light.Visibility = Visibility.Collapsed;
            }
        }

        private void ApplySearch(double Width)
        {
            SearchBox.Margin = new Thickness(0, 0, ((Width - SearchBox.MaxWidth) / 2) - 165, 0);
        }

        private void ApplyGeneral(bool Mode = true)
        {
            foreach (NavigationViewItem Menu in RootView.MenuItems)
            {
                if (Menu.Name.Contains("General"))
                {
                    Menu.Visibility = Visibility.Visible;
                }
                else
                {
                    Menu.Visibility = Visibility.Collapsed;
                }
            }

            FooterDock.Visibility = Visibility.Visible;
            Setting.Visibility = Visibility.Visible;

            if (Mode)
            {
                RootView.Navigate(typeof(SPVPLP));
            }
        }

        private void ApplySetting(bool Mode = true)
        {
            foreach (NavigationViewItem Menu in RootView.MenuItems)
            {
                if (Menu.Name.Contains("Setting"))
                {
                    Menu.Visibility = Visibility.Visible;
                }
                else
                {
                    Menu.Visibility = Visibility.Collapsed;
                }
            }

            FooterDock.Visibility = Visibility.Collapsed;
            Setting.Visibility = Visibility.Collapsed;

            if (Mode)
            {
                RootView.Navigate(typeof(SPVPSGSP));
            }
            else
            {
                ApplySearch(Width);
            }
        }

        private void ThemeChange_click(object sender, RoutedEventArgs e)
        {
            ApplyTheme(sender as Button);
        }

        private void NavigationChange_Click(object sender, RoutedEventArgs e)
        {
            NavigationViewItem View = sender as NavigationViewItem;

            if (View.Name == "Setting")
            {
                ApplySetting();
            }
            else
            {
                ApplyGeneral();
            }
        }

        private void MainWindow_SizeChanged(object sender, SizeChangedEventArgs e)
        {
            ApplySearch(e.NewSize.Width);
        }

        private void RootView_Navigated(NavigationView sender, NavigatedEventArgs args)
        {
            Dispose();
        }

        private void RootView_Navigating(NavigationView sender, NavigatingCancelEventArgs args)
        {
            Dispose();
        }

        public void Dispose()
        {
            RootView.ClearJournal();

            ViewModel.Dispose();

            GC.Collect();
            GC.SuppressFinalize(this);
        }
    }
}