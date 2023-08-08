using CommunityToolkit.Mvvm.ComponentModel;
using System.Windows.Threading;
using Wpf.Ui.Controls;
using SEOST = Skylark.Enum.OperatingSystemType;
using SMR = Sucrose.Memory.Readonly;
using SSCHA = Sucrose.Shared.Core.Helper.Architecture;
using SSCHF = Sucrose.Shared.Core.Helper.Framework;
using SSCHM = Sucrose.Shared.Core.Helper.Memory;
using SSCHOS = Sucrose.Shared.Core.Helper.OperatingSystem;
using SSCHV = Sucrose.Shared.Core.Helper.Version;
using SSRER = Sucrose.Shared.Resources.Extension.Resources;
using SMC = Sucrose.Memory.Constant;
using SMMI = Sucrose.Manager.Manage.Internal;

namespace Sucrose.Portal.ViewModels
{
    public partial class MainWindowViewModel : ObservableObject, INavigationAware, IDisposable
    {
        private static WindowBackdropType BackdropType => SMMI.PortalSettingManager.GetSetting(SMC.BackdropType, DefaultBackdropType);

        private static WindowBackdropType DefaultBackdropType => WindowBackdropType.None;

        [ObservableProperty]
        private WindowBackdropType _WindowBackdropType = DefaultBackdropType;

        private readonly DispatcherTimer Timer = new();

        [ObservableProperty]
        private SEOST _OperatingSystem = SEOST.Unknown;

        [ObservableProperty]
        private string _Architecture = string.Empty;

        [ObservableProperty]
        private string _Framework = string.Empty;

        [ObservableProperty]
        private string _Version = string.Empty;

        [ObservableProperty]
        private string _Quoting = string.Empty;

        [ObservableProperty]
        private string _Memory = string.Empty;

        private bool _isInitialized = false;

        public MainWindowViewModel()
        {
            if (!_isInitialized)
            {
                InitializeViewModel();

                Timer.Interval = TimeSpan.FromSeconds(1);
                Timer.Tick += Memory_Tick;
                Timer.Start();
            }
        }

        public void OnNavigatedTo() { }

        public void OnNavigatedFrom() { }

        private void InitializeViewModel()
        {
            Memory = SSCHM.Get();
            Quoting = GetQuoting();
            Version = SSCHV.GetText();
            Framework = SSCHF.GetName();
            Architecture = SSCHA.GetText();
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
                return BackdropType;
            }
            else
            {
                return DefaultBackdropType;
            }
        }

        private void Memory_Tick(object sender, EventArgs e)
        {
            Memory = SSCHM.Get();
        }

        public void Dispose()
        {
            GC.Collect();
            GC.SuppressFinalize(this);
        }
    }
}