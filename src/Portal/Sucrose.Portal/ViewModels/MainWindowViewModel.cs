using CommunityToolkit.Mvvm.ComponentModel;
using Wpf.Ui.Controls;
using SEAT = Skylark.Enum.AssemblyType;
using SEOST = Skylark.Enum.OperatingSystemType;
using SHV = Skylark.Helper.Versionly;
using SWHOS = Skylark.Wing.Helper.OperatingSystem;

namespace Sucrose.Portal.ViewModels
{
    public partial class MainWindowViewModel : ObservableObject, INavigationAware
    {
        private bool _isInitialized = false;

        [ObservableProperty]
        private string _Version = string.Empty;

        [ObservableProperty]
        private string _Framework = string.Empty;

        [ObservableProperty]
        private string _Architecture = string.Empty;

        [ObservableProperty]
        private SEOST _OperatingSystem = SEOST.Unknown;

        [ObservableProperty]
        private WindowBackdropType _WindowBackdropType = WindowBackdropType.None;

        public MainWindowViewModel()
        {
            if (!_isInitialized)
            {
                InitializeViewModel();
            }
        }

        public void OnNavigatedTo() { }

        public void OnNavigatedFrom() { }

        private void InitializeViewModel()
        {
            Version = GetVersion();
            Framework = GetFramework();
            Architecture = GetArchitecture();
            OperatingSystem = GetOperatingSystem();
            WindowBackdropType = GetWindowBackdropType();

            _isInitialized = true;
        }

        private string GetVersion()
        {
            return SHV.Auto(SEAT.Entry).ToString();
        }

        private string GetFramework()
        {
#if NET48
            return ".NET Framework 4.8";
#elif NET481
            return ".NET Framework 4.8.1";
#elif NET6_0
            return ".NET 6.0";
#elif NET7_0
            return ".NET 7.0";
#elif NET8_0
            return ".NET 8.0";
#else
            return "Unknown";
#endif
        }

        private string GetArchitecture()
        {
#if X64
            return "x64";
#elif X86
            return "x86";
#elif ARM64
            return "ARM64";
#else
            return "Unknown";
#endif
        }

        private SEOST GetOperatingSystem()
        {
            return SWHOS.GetSystem();
        }

        private WindowBackdropType GetWindowBackdropType()
        {
            if (OperatingSystem == SEOST.Windows11)
            {
                return WindowBackdropType.Acrylic;
            }
            else
            {
                return WindowBackdropType.Auto;
            }
        }
    }
}