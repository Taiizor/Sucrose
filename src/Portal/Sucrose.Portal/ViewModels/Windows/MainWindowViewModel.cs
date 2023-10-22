using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using System.IO;
using System.Windows;
using System.Windows.Media;
using System.Windows.Media.Imaging;
using System.Windows.Threading;
using Wpf.Ui.Controls;
using SEOST = Skylark.Enum.OperatingSystemType;
using SEWTT = Skylark.Enum.WindowsThemeType;
using SMC = Sucrose.Memory.Constant;
using SMMI = Sucrose.Manager.Manage.Internal;
using SMMM = Sucrose.Manager.Manage.Manager;
using SMR = Sucrose.Memory.Readonly;
using SPEIL = Sucrose.Portal.Extension.ImageLoader;
using SPMI = Sucrose.Portal.Manage.Internal;
using SPMM = Sucrose.Portal.Manage.Manager;
using SPVCDP = Sucrose.Portal.Views.Controls.DisplayPreferences;
using SPVCOA = Sucrose.Portal.Views.Controls.OtherAbout;
using SPVCOH = Sucrose.Portal.Views.Controls.OtherHelp;
using SPVCTC = Sucrose.Portal.Views.Controls.ThemeCreate;
using SSCHA = Sucrose.Shared.Core.Helper.Architecture;
using SSCHF = Sucrose.Shared.Core.Helper.Framework;
using SSCHM = Sucrose.Shared.Core.Helper.Memory;
using SSCHOS = Sucrose.Shared.Core.Helper.OperatingSystem;
using SSCHV = Sucrose.Shared.Core.Helper.Version;
using SSLHK = Sucrose.Shared.Live.Helper.Kill;
using SSLHR = Sucrose.Shared.Live.Helper.Run;
using SSRER = Sucrose.Shared.Resources.Extension.Resources;
using SSSHL = Sucrose.Shared.Space.Helper.Live;
using SSSHP = Sucrose.Shared.Space.Helper.Processor;
using SSSMI = Sucrose.Shared.Space.Manage.Internal;
using WUAAT = Wpf.Ui.Appearance.ApplicationTheme;
using WUAT = Wpf.Ui.Appearance.ApplicationThemeManager;

namespace Sucrose.Portal.ViewModels.Windows
{
    public partial class MainWindowViewModel : ObservableObject, IDisposable
    {
        [ObservableProperty]
        private WindowBackdropType _WindowBackdropType = GetWindowBackdropType();

        [ObservableProperty]
        private Stretch _Stretch = SPMM.DefaultBackgroundStretch;

        private readonly DispatcherTimer Timer = new();

        [ObservableProperty]
        private SEOST _OperatingSystem = SEOST.Unknown;

        [ObservableProperty]
        private string _Architecture = string.Empty;

        [ObservableProperty]
        private BitmapImage _Backgrounder2 = null;

        [ObservableProperty]
        private string _Framework = string.Empty;

        [ObservableProperty]
        private string _Version = string.Empty;

        [ObservableProperty]
        private string _Quoting = string.Empty;

        [ObservableProperty]
        private string _Memory = string.Empty;

        [ObservableProperty]
        private IconElement _ThemeIcon = null;

        private readonly SPEIL Loader = new();

        [ObservableProperty]
        private string _Backgrounder = null;

        [ObservableProperty]
        private double _Opacity = 1d;

        [ObservableProperty]
        private Visibility _Donater;

        private bool _isInitialized;

        public MainWindowViewModel()
        {
            if (!_isInitialized)
            {
                InitializeViewModel();

                Timer.Interval = TimeSpan.FromSeconds(1);
                Timer.Tick += Memory_Tick;
                Timer.Start();

                Information();
                Backdrop();
                Donate();
            }
        }

        private void Donate()
        {
            SPMI.DonateService.DonateVisibilityChanged += (s, e) => Donater = GetDonater();
        }

        private void Backdrop()
        {
            SPMI.BackdropService.BackdropOpacityChanged += (s, e) => Opacity = GetOpacity();
            SPMI.BackdropService.BackdropStretchChanged += (s, e) => Stretch = GetStretch();
            SPMI.BackdropService.BackdropImageChanged += (s, e) => Backgrounder = GetBackgrounder();
        }

        private void Information()
        {
            Dictionary<string, string> Information = new()
            {
                ["Version"] = Version,
                ["Framework"] = Framework,
                ["Architecture"] = Architecture,
                ["OperatingSystem"] = SSCHOS.GetText()
            };

            SMMI.CoreSettingManager.SetSetting(SMC.Information, Information);
        }

        private void InitializeViewModel()
        {
            Memory = SSCHM.Get();
            Donater = GetDonater();
            Quoting = GetQuoting();
            Stretch = GetStretch();
            Opacity = GetOpacity();
            Version = SSCHV.GetText();
            Framework = SSCHF.GetName();
            Architecture = SSCHA.GetText();
            OperatingSystem = SSCHOS.Get();
            Backgrounder = GetBackgrounder();
            WindowBackdropType = GetWindowBackdropType();

            _isInitialized = true;
        }

        private double GetOpacity()
        {
            return SMMM.BackgroundOpacity / 100d;
        }

        private string GetQuoting()
        {
            return SSRER.GetValue("Portal", $"Quoting{SMR.Randomise.Next(86)}");
        }

        private Stretch GetStretch()
        {
            Stretch Type = SPMM.BackgroundStretch;

            if ((int)Type < Enum.GetValues(typeof(Stretch)).Length)
            {
                return Type;
            }
            else
            {
                return SPMM.DefaultBackgroundStretch;
            }
        }

        private Visibility GetDonater()
        {
            return SMMM.DonateVisible ? Visibility.Visible : Visibility.Collapsed;
        }

        private string GetBackgrounder()
        {
            if (File.Exists(SMMM.BackgroundImage))
            {
                return SMMM.BackgroundImage;
            }
            else
            {
                return null;
            }
        }

        private BitmapImage GetBackgrounder2()
        {
            if (File.Exists(SMMM.BackgroundImage))
            {
                return Loader.LoadOptimal(SMMM.BackgroundImage, false);
            }
            else
            {
                return null;
            }
        }

        private static WindowBackdropType GetWindowBackdropType()
        {
            if (WindowBackdrop.IsSupported(SPMM.BackdropType))
            {
                return SPMM.BackdropType;
            }
            else
            {
                return SPMM.DefaultBackdropType;
            }
        }

        [RelayCommand]
        private void OnChangeTheme()
        {
            if (SPMM.ThemeType == SEWTT.Dark)
            {
                SMMI.GeneralSettingManager.SetSetting(SMC.ThemeType, SEWTT.Light);
                WUAT.Apply(WUAAT.Light, GetWindowBackdropType(), true, true);
            }
            else
            {
                SMMI.GeneralSettingManager.SetSetting(SMC.ThemeType, SEWTT.Dark);
                WUAT.Apply(WUAAT.Dark, GetWindowBackdropType(), true, true);
            }

            if (GetWindowBackdropType() == WindowBackdropType.None)
            {
                WindowBackdrop.RemoveBackdrop(Application.Current.MainWindow);
            }
        }

        [RelayCommand]
        private async Task OnOtherHelp()
        {
            SPVCOH OtherHelp = new();

            await OtherHelp.ShowAsync();

            OtherHelp.Dispose();
        }

        [RelayCommand]
        private async Task OnOtherAbout()
        {
            SPVCOA OtherAbout = new();

            await OtherAbout.ShowAsync();

            OtherAbout.Dispose();
        }

        [RelayCommand]
        private async Task OnCreateWallpaper()
        {
            SPVCTC ThemeCreate = new();

            ContentDialogResult Result = await ThemeCreate.ShowAsync();

            if (Result == ContentDialogResult.Primary)
            {
                if ((!SMMM.ClosePerformance && !SMMM.PausePerformance) || !SSSHP.Work(SSSMI.Backgroundog))
                {
                    if (SMMM.LibraryStart)
                    {
                        SMMI.LibrarySettingManager.SetSetting(SMC.LibrarySelected, SPMI.LibraryService.Theme);

                        if (SSSHL.Run())
                        {
                            SSLHK.Stop();
                        }

                        SSLHR.Start();
                    }
                }

                SPMI.LibraryService.CreateWallpaper();
            }

            ThemeCreate.Dispose();
        }

        [RelayCommand]
        private async Task OnDisplayPreferences()
        {
            SPVCDP DisplayPreferences = new();

            await DisplayPreferences.ShowAsync();

            DisplayPreferences.Dispose();
        }

        private void Memory_Tick(object sender, EventArgs e)
        {
            Memory = SSCHM.Get();
            Dispose();
        }

        public void Dispose()
        {
            Loader.Dispose();

            GC.Collect();
            GC.SuppressFinalize(this);
        }
    }
}