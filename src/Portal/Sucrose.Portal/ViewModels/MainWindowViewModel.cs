using CommunityToolkit.Mvvm.ComponentModel;
using Wpf.Ui.Controls;
using SEOST = Skylark.Enum.OperatingSystemType;
using SMR = Sucrose.Memory.Readonly;
using SSRER = Sucrose.Shared.Resources.Extension.Resources;
using SSCHV = Sucrose.Shared.Core.Helper.Version;
using SSCHF = Sucrose.Shared.Core.Helper.Framework;
using SSCHA = Sucrose.Shared.Core.Helper.Architecture;
using SSCHOS = Sucrose.Shared.Core.Helper.OperatingSystem;

namespace Sucrose.Portal.ViewModels
{
    public partial class MainWindowViewModel : ObservableObject, INavigationAware
    {
        private bool _isInitialized = false;

        [ObservableProperty]
        private string _Quoting = string.Empty;

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
            Quoting = GetQuoting();
            Version = SSCHV.GetText();
            Framework = SSCHF.Get();
            Architecture = SSCHA.Get();
            OperatingSystem = SSCHOS.Get();
            WindowBackdropType = GetWindowBackdropType();

            _isInitialized = true;
        }

        private string GetQuoting()
        {
            return SSRER.GetValue("Portal", $"Quoting{SMR.Randomise.Next(40)}");
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