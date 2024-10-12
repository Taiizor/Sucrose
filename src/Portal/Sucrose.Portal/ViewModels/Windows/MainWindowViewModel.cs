using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using System.IO;
using System.Windows;
using System.Windows.Media;
using System.Windows.Media.Imaging;
using System.Windows.Threading;
using Wpf.Ui;
using Wpf.Ui.Controls;
using SEOST = Skylark.Enum.OperatingSystemType;
using SEWTT = Skylark.Enum.WindowsThemeType;
using SMC = Sucrose.Memory.Constant;
using SMMCC = Sucrose.Memory.Manage.Constant.Core;
using SMMCG = Sucrose.Memory.Manage.Constant.General;
using SMMI = Sucrose.Manager.Manage.Internal;
using SMMM = Sucrose.Manager.Manage.Manager;
using SMR = Sucrose.Memory.Readonly;
using SPEIL = Sucrose.Portal.Extension.ImageLoader;
using SPMI = Sucrose.Portal.Manage.Internal;
using SPMMP = Sucrose.Portal.Manage.Manager.Portal;
using SPVCDP = Sucrose.Portal.Views.Controls.DisplayPreferences;
using SPVCOA = Sucrose.Portal.Views.Controls.OtherAbout;
using SPVCOH = Sucrose.Portal.Views.Controls.OtherHelp;
using SPVCTC = Sucrose.Portal.Views.Controls.ThemeCreate;
using SPVCWC = Sucrose.Portal.Views.Controls.WallpaperCycling;
using SRER = Sucrose.Resources.Extension.Resources;
using SSCHA = Sucrose.Shared.Core.Helper.Architecture;
using SSCHF = Sucrose.Shared.Core.Helper.Framework;
using SSCHM = Sucrose.Shared.Core.Helper.Memory;
using SSCHOS = Sucrose.Shared.Core.Helper.OperatingSystem;
using SSCHV = Sucrose.Shared.Core.Helper.Version;
using SSDMI = Sucrose.Shared.Dependency.Manage.Internal;
using SSDMMG = Sucrose.Shared.Dependency.Manage.Manager.General;
using SSDMMP = Sucrose.Shared.Dependency.Manage.Manager.Portal;
using SSLHK = Sucrose.Shared.Live.Helper.Kill;
using SSLHR = Sucrose.Shared.Live.Helper.Run;
using SSSHL = Sucrose.Shared.Space.Helper.Live;
using SSSHP = Sucrose.Shared.Space.Helper.Processor;
using SSSMI = Sucrose.Shared.Space.Manage.Internal;
using SWHSI = Skylark.Wing.Helper.SystemInfo;
using WUAAT = Wpf.Ui.Appearance.ApplicationTheme;
using WUAT = Wpf.Ui.Appearance.ApplicationThemeManager;
using SMML = Sucrose.Manager.Manage.Library;
using SMMCL = Sucrose.Memory.Manage.Constant.Library;

namespace Sucrose.Portal.ViewModels.Windows
{
    public partial class MainWindowViewModel : ViewModel, IDisposable
    {
        [ObservableProperty]
        private WindowBackdropType _WindowBackdropType = GetWindowBackdropType();

        private readonly IContentDialogService _ContentDialogService;

        [ObservableProperty]
        private Stretch _Stretch = SSDMI.DefaultBackgroundStretch;

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

        public MainWindowViewModel(IContentDialogService contentDialogService)
        {
            if (!_isInitialized)
            {
                InitializeViewModel();

                _ContentDialogService = contentDialogService;

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
                ["OperatingSystem"] = SSCHOS.GetText(),
                ["OperatingSystemBuild"] = SSCHV.GetOSText(),
                ["ProcessArchitecture"] = SSCHOS.GetProcessArchitectureText(),
                ["ProcessorArchitecture"] = SSCHOS.GetProcessorArchitecture(),
                ["OperatingSystemArchitecture"] = SWHSI.GetSystemInfoArchitecture()
            };

            SMMI.CoreSettingManager.SetSetting(SMMCC.Information, Information);
        }

        private void InitializeViewModel()
        {
            Donater = GetDonater();
            Quoting = GetQuoting();
            Stretch = GetStretch();
            Opacity = GetOpacity();
            Version = SSCHV.GetText();
            Framework = SSCHF.GetName();
            Architecture = SSCHA.GetText();
            OperatingSystem = SSCHOS.Get();
            Backgrounder = GetBackgrounder();
            Memory = SSCHM.GetCurrentProcess();
            WindowBackdropType = GetWindowBackdropType();

            _isInitialized = true;
        }

        private double GetOpacity()
        {
            return SMMM.BackgroundOpacity / 100d;
        }

        private string GetQuoting()
        {
            return SRER.GetValue("Portal", $"Quoting{SMR.Randomise.Next(86)}");
        }

        private Stretch GetStretch()
        {
            Stretch Type = SSDMMP.BackgroundStretch;

            if ((int)Type < Enum.GetValues(typeof(Stretch)).Length)
            {
                return Type;
            }
            else
            {
                return SSDMI.DefaultBackgroundStretch;
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
            if (WindowBackdrop.IsSupported(SPMMP.BackdropType))
            {
                return SPMMP.BackdropType;
            }
            else
            {
                return SPMI.DefaultBackdropType;
            }
        }

        [RelayCommand]
        private void OnChangeTheme()
        {
            if (SSDMMG.ThemeType == SEWTT.Dark)
            {
                SMMI.GeneralSettingManager.SetSetting(SMMCG.ThemeType, SEWTT.Light);
                WUAT.Apply(WUAAT.Light, GetWindowBackdropType(), true);
            }
            else
            {
                SMMI.GeneralSettingManager.SetSetting(SMMCG.ThemeType, SEWTT.Dark);
                WUAT.Apply(WUAAT.Dark, GetWindowBackdropType(), true);
            }
        }

        [RelayCommand]
        private async Task OnOtherHelp()
        {
            SPVCOH OtherHelp = new(_ContentDialogService.GetDialogHost());

            await OtherHelp.ShowAsync();

            OtherHelp.Dispose();
        }

        [RelayCommand]
        private async Task OnOtherAbout()
        {
            SPVCOA OtherAbout = new(_ContentDialogService.GetDialogHost());

            await OtherAbout.ShowAsync();

            OtherAbout.Dispose();
        }

        [RelayCommand]
        private async Task OnCreateWallpaper()
        {
            SPVCTC ThemeCreate = new(_ContentDialogService.GetDialogHost());

            ContentDialogResult Result = await ThemeCreate.ShowAsync();

            if (Result == ContentDialogResult.Primary)
            {
                if ((!SMMM.ClosePerformance && !SMMM.PausePerformance) || !SSSHP.Work(SSSMI.Backgroundog))
                {
                    if (SMMM.LibraryStart)
                    {
                        SMMI.LibrarySettingManager.SetSetting(SMMCL.LibrarySelected, SPMI.LibraryService.Theme);

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
        private async Task OnWallpaperCycling()
        {
            SPVCWC WallpaperCycling = new(_ContentDialogService.GetDialogHost());

            await WallpaperCycling.ShowAsync();

            WallpaperCycling.Dispose();
        }

        [RelayCommand]
        private async Task OnDisplayPreferences()
        {
            SPVCDP DisplayPreferences = new(_ContentDialogService.GetDialogHost());

            await DisplayPreferences.ShowAsync();

            DisplayPreferences.Dispose();
        }

        private void Memory_Tick(object sender, EventArgs e)
        {
            Memory = SSCHM.GetCurrentProcess();
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