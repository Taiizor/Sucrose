using System.Windows;
using System.Windows.Controls;
using Wpf.Ui.Contracts;
using Wpf.Ui.Controls;
using Button = Wpf.Ui.Controls.Button;
using SEWTT = Skylark.Enum.WindowsThemeType;
using SMC = Sucrose.Memory.Constant;
using SMMI = Sucrose.Manager.Manage.Internal;
using SPMI = Sucrose.Portal.Manage.Internal;
using SPSCIW = Sucrose.Portal.Services.Contracts.IWindow;
using SPVMWMWVM = Sucrose.Portal.ViewModels.Windows.MainWindowViewModel;
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
    public partial class MainWindow : SPSCIW, IDisposable
    {
        private static SEWTT Theme => SMMI.GeneralSettingManager.GetSetting(SMC.ThemeType, SWHWT.GetTheme());

        public SPVMWMWVM ViewModel { get; }

        public MainWindow(SPVMWMWVM ViewModel, INavigationService NavigationService, IServiceProvider ServiceProvider, ISnackbarService SnackbarService, IContentDialogService ContentDialogService)
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

            SPMI.ServiceProvider = ServiceProvider;
            SPMI.SnackbarService = SnackbarService;
            SPMI.NavigationService = NavigationService;
            SPMI.ContentDialogService = ContentDialogService;

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
            SearchBox.Margin = new Thickness(0, 0, ((Width - SearchBox.MaxWidth) / 2) - 195, 0);
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

            Dispose();
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

        private void SearchBox_TextChanged(object sender, TextChangedEventArgs e)
        {
            SPMI.SearchService.SearchText = SearchBox.Text;

            Dispose();
        }

        private void MainWindow_SizeChanged(object sender, SizeChangedEventArgs e)
        {
            ApplySearch(e.NewSize.Width);

            Dispose();
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