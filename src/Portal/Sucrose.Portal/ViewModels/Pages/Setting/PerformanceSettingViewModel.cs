using CommunityToolkit.Mvvm.ComponentModel;
using System.Windows;
using System.Windows.Controls;
using System.Windows.Media;
using Wpf.Ui.Controls;
using SEST = Skylark.Enum.StorageType;
using SMC = Sucrose.Memory.Constant;
using SMMI = Sucrose.Manager.Manage.Internal;
using SMMM = Sucrose.Manager.Manage.Manager;
using SPMM = Sucrose.Portal.Manage.Manager;
using SMR = Sucrose.Memory.Readonly;
using SPVCEC = Sucrose.Portal.Views.Controls.ExpanderCard;
using SSDECT = Sucrose.Shared.Dependency.Enum.CommandsType;
using SSDEPT = Sucrose.Shared.Dependency.Enum.PerformanceType;
using SSRER = Sucrose.Shared.Resources.Extension.Resources;
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

            SPVCEC Network = new()
            {
                Margin = new Thickness(0, 10, 0, 0)
            };

            Network.Title.Text = "Ağ Kullanımı";
            Network.LeftIcon.Symbol = SymbolRegular.NetworkCheck24;
            Network.Description.Text = "Ağ kullanımı belirlenen sınırı geçtiğinde duvar kağıdına ne olacağı.";

            ComboBox NetworkPerformance = new();

            NetworkPerformance.SelectionChanged += (s, e) => NetworkPerformanceSelected(NetworkPerformance.SelectedIndex);

            foreach (SSDEPT Type in Enum.GetValues(typeof(SSDEPT)))
            {
                NetworkPerformance.Items.Add(Type);
            }

            NetworkPerformance.SelectedIndex = (int)SPMM.NetworkPerformance;

            Network.HeaderFrame = NetworkPerformance;

            StackPanel NetworkContent = new();

            StackPanel NetworkAdapterContent = new()
            {
                Orientation = Orientation.Horizontal
            };

            TextBlock NetworkAdapterText = new()
            {
                Foreground = SSRER.GetResource<Brush>("TextFillColorPrimaryBrush"),
                VerticalAlignment = VerticalAlignment.Center,
                Margin = new Thickness(0, 0, 10, 0),
                FontWeight = FontWeights.SemiBold,
                Text = "Ağ Adaptörü:"
            };

            ComboBox NetworkAdapter = new()
            {
                MaxWidth = 700
            };

            NetworkAdapter.SelectionChanged += (s, e) => NetworkAdapterSelected($"{NetworkAdapter.SelectedValue}");

            foreach (string Interface in SMMM.NetworkInterfaces)
            {
                NetworkAdapter.Items.Add(Interface);
            }

            NetworkAdapter.SelectedValue = SMMM.NetworkAdapter;

            StackPanel NetworkUploadContent = new()
            {
                Orientation = Orientation.Horizontal,
                Margin = new Thickness(0, 10, 0, 0)
            };

            TextBlock NetworkUploadText = new()
            {
                Foreground = SSRER.GetResource<Brush>("TextFillColorPrimaryBrush"),
                VerticalAlignment = VerticalAlignment.Center,
                Margin = new Thickness(0, 0, 10, 0),
                FontWeight = FontWeights.SemiBold,
                Text = "Yükleme Boyutu:"
            };

            NumberBox NetworkUpload = new()
            {
                Margin = new Thickness(0, 0, 10, 0),
                ClearButtonEnabled = false,
                Value = SMMM.UploadValue,
                MaxDecimalPlaces = 0,
                Maximum = 99999999,
                MaxLength = 8,
                Minimum = 0
            };

            ComboBox NetworkUploadType = new();

            NetworkUploadType.SelectionChanged += (s, e) => NetworkUploadTypeSelected(NetworkUploadType.SelectedIndex);

            foreach (SEST Type in Enum.GetValues(typeof(SEST)))
            {
                NetworkUploadType.Items.Add(Type);
            }

            NetworkUploadType.SelectedIndex = (int)SMMM.UploadType;

            StackPanel NetworkDownloadContent = new()
            {
                Orientation = Orientation.Horizontal,
                Margin = new Thickness(0, 10, 0, 0)
            };

            TextBlock NetworkDownloadText = new()
            {
                Foreground = SSRER.GetResource<Brush>("TextFillColorPrimaryBrush"),
                VerticalAlignment = VerticalAlignment.Center,
                Margin = new Thickness(0, 0, 10, 0),
                FontWeight = FontWeights.SemiBold,
                Text = "İndirme Boyutu:"
            };

            NumberBox NetworkDownload = new()
            {
                Margin = new Thickness(0, 0, 10, 0),
                ClearButtonEnabled = false,
                Value = SMMM.DownloadValue,
                MaxDecimalPlaces = 0,
                Maximum = 99999999,
                MaxLength = 8,
                Minimum = 0
            };

            ComboBox NetworkDownloadType = new();

            NetworkDownloadType.SelectionChanged += (s, e) => NetworkDownloadTypeSelected(NetworkDownloadType.SelectedIndex);

            foreach (SEST Type in Enum.GetValues(typeof(SEST)))
            {
                NetworkDownloadType.Items.Add(Type);
            }

            NetworkDownloadType.SelectedIndex = (int)SMMM.DownloadType;

            NetworkAdapterContent.Children.Add(NetworkAdapterText);
            NetworkAdapterContent.Children.Add(NetworkAdapter);

            NetworkUploadContent.Children.Add(NetworkUploadText);
            NetworkUploadContent.Children.Add(NetworkUpload);
            NetworkUploadContent.Children.Add(NetworkUploadType);

            NetworkDownloadContent.Children.Add(NetworkDownloadText);
            NetworkDownloadContent.Children.Add(NetworkDownload);
            NetworkDownloadContent.Children.Add(NetworkDownloadType);

            NetworkContent.Children.Add(NetworkAdapterContent);
            NetworkContent.Children.Add(NetworkUploadContent);
            NetworkContent.Children.Add(NetworkDownloadContent);

            Network.FooterCard = NetworkContent;

            Contents.Add(Network);

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

        private void NetworkUploadTypeSelected(int Index)
        {
            if (Index != (int)SMMM.UploadType)
            {
                SEST Type = (SEST)Index;

                SMMI.BackgroundogSettingManager.SetSetting(SMC.UploadType, Type);
            }
        }

        private void NetworkAdapterSelected(string Value)
        {
            if (Value != SMMM.NetworkAdapter)
            {
                SMMI.BackgroundogSettingManager.SetSetting(SMC.NetworkAdapter, Value);
            }
        }

        private void NetworkPerformanceSelected(int Index)
        {
            if (Index != (int)SPMM.NetworkPerformance)
            {
                SSDEPT Type = (SSDEPT)Index;

                SMMI.BackgroundogSettingManager.SetSetting(SMC.NetworkPerformance, Type);
            }
        }

        private void NetworkDownloadTypeSelected(int Index)
        {
            if (Index != (int)SMMM.DownloadType)
            {
                SEST Type = (SEST)Index;

                SMMI.BackgroundogSettingManager.SetSetting(SMC.DownloadType, Type);
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