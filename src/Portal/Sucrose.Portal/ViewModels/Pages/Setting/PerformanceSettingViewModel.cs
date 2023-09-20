using CommunityToolkit.Mvvm.ComponentModel;
using System.Windows;
using System.Windows.Controls;
using System.Windows.Media;
using Wpf.Ui.Controls;
using SMC = Sucrose.Memory.Constant;
using SMMI = Sucrose.Manager.Manage.Internal;
using SMMM = Sucrose.Manager.Manage.Manager;
using SMR = Sucrose.Memory.Readonly;
using SPVCEC = Sucrose.Portal.Views.Controls.ExpanderCard;
using SSDECT = Sucrose.Shared.Dependency.Enum.CommandsType;
using SSRER = Sucrose.Shared.Resources.Extension.Resources;
using SSSHN = Sucrose.Shared.Space.Helper.Network;
using SSSHP = Sucrose.Shared.Space.Helper.Processor;
using SSSMI = Sucrose.Shared.Space.Manage.Internal;
using TextBlock = System.Windows.Controls.TextBlock;

namespace Sucrose.Portal.ViewModels.Pages
{
    public partial class PerformanceSettingViewModel : ObservableObject, INavigationAware, IDisposable
    {
        [ObservableProperty]
        private List<UIElement> _Contents = new();

        private bool _isInitialized;

        public PerformanceSettingViewModel()
        {
            if (!_isInitialized)
            {
                InitializeViewModel();
            }
        }

        private void InitializeViewModel()
        {
            TextBlock AppearanceBehaviorArea = new()
            {
                Foreground = SSRER.GetResource<Brush>("TextFillColorPrimaryBrush"),
                Margin = new Thickness(0, 0, 0, 0),
                FontWeight = FontWeights.Bold,
                Text = "Görünüş & Davranış"
            };

            Contents.Add(AppearanceBehaviorArea);

            SPVCEC Counter = new()
            {
                Margin = new Thickness(0, 10, 0, 0),
                Expandable = false
            };

            Counter.Title.Text = "Performans Sayaçları";
            Counter.LeftIcon.Symbol = SymbolRegular.ShiftsActivity24;
            Counter.Description.Text = "Tüm performans sayaçlarının arkaplanda çalışıp çalışmayacağı.";

            ToggleSwitch CounterState = new()
            {
                IsChecked = SMMM.PerformanceCounter
            };

            CounterState.Checked += (s, e) => CounterStateChecked(true);
            CounterState.Unchecked += (s, e) => CounterStateChecked(false);

            Counter.HeaderFrame = CounterState;

            Contents.Add(Counter);

            TextBlock SystemResourcesArea = new()
            {
                Foreground = SSRER.GetResource<Brush>("TextFillColorPrimaryBrush"),
                Margin = new Thickness(0, 10, 0, 0),
                FontWeight = FontWeights.Bold,
                Text = "Sistem Kaynakları"
            };

            Contents.Add(SystemResourcesArea);

            SPVCEC Adapter = new()
            {
                Margin = new Thickness(0, 10, 0, 0)
            };

            Adapter.Title.Text = "Ağ Kullanımı";
            Adapter.LeftIcon.Symbol = SymbolRegular.NetworkCheck24;
            Adapter.Description.Text = "Ağ kullanımı belirlenen sınırı geçtiğinde duvar kağıdına ne olacağı.";

            ComboBox NetworkAdapter = new()
            {
                MaxWidth = 250
            };

            NetworkAdapter.SelectionChanged += (s, e) => NetworkAdapterSelected($"{NetworkAdapter.SelectedValue}");

            foreach (string Network in SSSHN.InstanceNetworkInterfaces())
            {
                NetworkAdapter.Items.Add(Network);
            }

            NetworkAdapter.SelectedValue = SMMM.NetworkAdapter;

            Adapter.HeaderFrame = NetworkAdapter;

            StackPanel AdapterContent = new();

            StackPanel AdapterUploadContent = new()
            {
                Orientation = Orientation.Horizontal
            };

            TextBlock AdapterUploadText = new()
            {
                Foreground = SSRER.GetResource<Brush>("TextFillColorPrimaryBrush"),
                VerticalAlignment = VerticalAlignment.Center,
                Margin = new Thickness(0, 0, 10, 0),
                FontWeight = FontWeights.SemiBold,
                Text = "Yükleme Boyutu:"
            };

            NumberBox AdapterUpload = new()
            {
                ClearButtonEnabled = false,
                Value = SMMM.UploadValue,
                MaxDecimalPlaces = 0,
                Maximum = 99999999,
                MaxLength = 8,
                Minimum = 0
            };

            StackPanel AdapterDownloadContent = new()
            {
                Orientation = Orientation.Horizontal,
                Margin = new Thickness(0, 10, 0, 0)
            };

            TextBlock AdapterDownloadText = new()
            {
                Foreground = SSRER.GetResource<Brush>("TextFillColorPrimaryBrush"),
                VerticalAlignment = VerticalAlignment.Center,
                Margin = new Thickness(0, 0, 10, 0),
                FontWeight = FontWeights.SemiBold,
                Text = "İndirme Boyutu:"
            };

            NumberBox AdapterDownload = new()
            {
                ClearButtonEnabled = false,
                Value = SMMM.DownloadValue,
                MaxDecimalPlaces = 0,
                Maximum = 99999999,
                MaxLength = 8,
                Minimum = 0
            };

            AdapterUploadContent.Children.Add(AdapterUploadText);
            AdapterUploadContent.Children.Add(AdapterUpload);

            AdapterDownloadContent.Children.Add(AdapterDownloadText);
            AdapterDownloadContent.Children.Add(AdapterDownload);

            AdapterContent.Children.Add(AdapterUploadContent);
            AdapterContent.Children.Add(AdapterDownloadContent);

            Adapter.FooterCard = AdapterContent;

            Contents.Add(Adapter);

            _isInitialized = true;
        }

        public void OnNavigatedTo()
        {
            //
        }

        public void OnNavigatedFrom()
        {
            //Dispose();
        }

        private void CounterStateChecked(bool State)
        {
            SMMI.BackgroundogSettingManager.SetSetting(SMC.PerformanceCounter, State);

            if (State)
            {
                SSSHP.Run(SSSMI.Commandog, $"{SMR.StartCommand}{SSDECT.Backgroundog}{SMR.ValueSeparator}{SSSMI.Backgroundog}");
            }
            else
            {
                if (SSSHP.Work(SMR.Backgroundog))
                {
                    SSSHP.Kill(SMR.Backgroundog);
                }
            }
        }

        private void NetworkAdapterSelected(string Value)
        {
            if (Value != SMMM.NetworkAdapter)
            {
                SMMI.BackgroundogSettingManager.SetSetting(SMC.NetworkAdapter, Value);
            }
        }

        public void Dispose()
        {
            Contents.Clear();

            GC.Collect();
            GC.SuppressFinalize(this);
        }
    }
}