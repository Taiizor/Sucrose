using CommunityToolkit.Mvvm.ComponentModel;
using System.Windows;
using System.Windows.Controls;
using System.Windows.Media;
using Wpf.Ui.Controls;
using SEST = Skylark.Enum.StorageType;
using SMC = Sucrose.Memory.Constant;
using SMMI = Sucrose.Manager.Manage.Internal;
using SMMM = Sucrose.Manager.Manage.Manager;
using SMR = Sucrose.Memory.Readonly;
using SPMM = Sucrose.Portal.Manage.Manager;
using SPVCEC = Sucrose.Portal.Views.Controls.ExpanderCard;
using SSDECT = Sucrose.Shared.Dependency.Enum.CommandsType;
using SSDEPT = Sucrose.Shared.Dependency.Enum.PerformanceType;
using SSDSHHS = Sucrose.Shared.Dependency.Struct.Host.HostStruct;
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
                Expandable = !SMMM.PerformanceCounter,
                IsExpand = !SMMM.PerformanceCounter,
                Margin = new Thickness(0, 10, 0, 0)
            };

            Counter.Title.Text = "Performans Sayaçları";
            Counter.LeftIcon.Symbol = SymbolRegular.ShiftsActivity24;
            Counter.Description.Text = "Tüm performans sayaçlarının arkaplanda çalışıp çalışmayacağı.";

            ToggleSwitch CounterState = new()
            {
                IsChecked = SMMM.PerformanceCounter
            };

            CounterState.Checked += (s, e) => CounterStateChecked(Counter, true);
            CounterState.Unchecked += (s, e) => CounterStateChecked(Counter, false);

            TextBlock CounterHint = new()
            {
                Text = "Not: Bazı Web türündeki temalar düzgün çalışmayabilir ve aşağıdaki performans ayarlarının hiç birisi çalışmayacaktır.",
                Foreground = SSRER.GetResource<Brush>("TextFillColorSecondaryBrush"),
                HorizontalAlignment = HorizontalAlignment.Left,
                TextWrapping = TextWrapping.WrapWithOverflow,
                Margin = new Thickness(0, 0, 0, 0),
                TextAlignment = TextAlignment.Left,
                FontWeight = FontWeights.SemiBold
            };

            Counter.FooterCard = CounterHint;

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

            SPVCEC Cpu = new()
            {
                Margin = new Thickness(0, 10, 0, 0)
            };

            Cpu.Title.Text = "İşemci Kullanımı";
            Cpu.LeftIcon.Symbol = SymbolRegular.HeartPulse24;
            Cpu.Description.Text = "İşlemci kullanımı ayarlarınız sonucunda duvar kağıdına ne olacağı.";

            ComboBox CpuPerformance = new();

            CpuPerformance.SelectionChanged += (s, e) => CpuPerformanceSelected(CpuPerformance.SelectedIndex);

            foreach (SSDEPT Type in Enum.GetValues(typeof(SSDEPT)))
            {
                CpuPerformance.Items.Add(Type);
            }

            CpuPerformance.SelectedIndex = (int)SPMM.CpuPerformance;

            Cpu.HeaderFrame = CpuPerformance;

            StackPanel CpuContent = new()
            {
                Orientation = Orientation.Horizontal
            };

            TextBlock CpuUsageText = new()
            {
                Foreground = SSRER.GetResource<Brush>("TextFillColorPrimaryBrush"),
                VerticalAlignment = VerticalAlignment.Center,
                Margin = new Thickness(0, 0, 10, 0),
                FontWeight = FontWeights.SemiBold,
                Text = "İşlemci Kullanımı (%):"
            };

            NumberBox CpuUsage = new()
            {
                Margin = new Thickness(0, 0, 10, 0),
                ClearButtonEnabled = false,
                Value = SMMM.CpuUsage,
                MaxDecimalPlaces = 0,
                Maximum = 100,
                MaxLength = 3,
                Minimum = 0
            };

            CpuUsage.ValueChanged += (s, e) => CpuUsageChanged(CpuUsage.Value);

            CpuContent.Children.Add(CpuUsageText);
            CpuContent.Children.Add(CpuUsage);

            Cpu.FooterCard = CpuContent;

            Contents.Add(Cpu);

            SPVCEC Memory = new()
            {
                Margin = new Thickness(0, 10, 0, 0)
            };

            Memory.Title.Text = "Bellek Kullanımı";
            Memory.LeftIcon.Symbol = SymbolRegular.Memory16;
            Memory.Description.Text = "Bellek kullanımı ayarlarınız sonucunda duvar kağıdına ne olacağı.";

            ComboBox MemoryPerformance = new();

            MemoryPerformance.SelectionChanged += (s, e) => MemoryPerformanceSelected(MemoryPerformance.SelectedIndex);

            foreach (SSDEPT Type in Enum.GetValues(typeof(SSDEPT)))
            {
                MemoryPerformance.Items.Add(Type);
            }

            MemoryPerformance.SelectedIndex = (int)SPMM.MemoryPerformance;

            Memory.HeaderFrame = MemoryPerformance;

            StackPanel MemoryContent = new()
            {
                Orientation = Orientation.Horizontal
            };

            TextBlock MemoryUsageText = new()
            {
                Foreground = SSRER.GetResource<Brush>("TextFillColorPrimaryBrush"),
                VerticalAlignment = VerticalAlignment.Center,
                Margin = new Thickness(0, 0, 10, 0),
                FontWeight = FontWeights.SemiBold,
                Text = "Bellek Kullanımı (%):"
            };

            NumberBox MemoryUsage = new()
            {
                Margin = new Thickness(0, 0, 10, 0),
                ClearButtonEnabled = false,
                Value = SMMM.MemoryUsage,
                MaxDecimalPlaces = 0,
                Maximum = 100,
                MaxLength = 3,
                Minimum = 0
            };

            MemoryUsage.ValueChanged += (s, e) => MemoryUsageChanged(MemoryUsage.Value);

            MemoryContent.Children.Add(MemoryUsageText);
            MemoryContent.Children.Add(MemoryUsage);

            Memory.FooterCard = MemoryContent;

            Contents.Add(Memory);

            SPVCEC Network = new()
            {
                Margin = new Thickness(0, 10, 0, 0)
            };

            Network.Title.Text = "Ağ Kullanımı";
            Network.LeftIcon.Symbol = SymbolRegular.NetworkCheck24;
            Network.Description.Text = "Ağ kullanımı ayarlarınız sonucunda duvar kağıdına ne olacağı.";

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

            NetworkUpload.ValueChanged += (s, e) => NetworkUploadChanged(NetworkUpload.Value);

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

            NetworkDownload.ValueChanged += (s, e) => NetworkDownloadChanged(NetworkDownload.Value);

            ComboBox NetworkDownloadType = new();

            NetworkDownloadType.SelectionChanged += (s, e) => NetworkDownloadTypeSelected(NetworkDownloadType.SelectedIndex);

            foreach (SEST Type in Enum.GetValues(typeof(SEST)))
            {
                NetworkDownloadType.Items.Add(Type);
            }

            NetworkDownloadType.SelectedIndex = (int)SMMM.DownloadType;

            StackPanel NetworkPingContent = new()
            {
                Orientation = Orientation.Horizontal,
                Margin = new Thickness(0, 10, 0, 0)
            };

            TextBlock NetworkPingText = new()
            {
                Foreground = SSRER.GetResource<Brush>("TextFillColorPrimaryBrush"),
                VerticalAlignment = VerticalAlignment.Center,
                Margin = new Thickness(0, 0, 10, 0),
                FontWeight = FontWeights.SemiBold,
                Text = "Ping Değeri (MS):"
            };

            NumberBox NetworkPing = new()
            {
                Margin = new Thickness(0, 0, 10, 0),
                ClearButtonEnabled = false,
                Value = SMMM.PingValue,
                MaxDecimalPlaces = 0,
                Maximum = 10000,
                MaxLength = 5,
                Minimum = 0
            };

            NetworkPing.ValueChanged += (s, e) => NetworkPingChanged(NetworkPing.Value);

            ComboBox NetworkPingType = new();

            NetworkPingType.SelectionChanged += (s, e) => NetworkPingTypeSelected($"{NetworkPingType.SelectedValue}");

            foreach (SSDSHHS Host in SSSHN.GetHost())
            {
                NetworkPingType.Items.Add(Host.Name);
            }

            NetworkPingType.SelectedValue = SMMM.PingType;

            NetworkAdapterContent.Children.Add(NetworkAdapterText);
            NetworkAdapterContent.Children.Add(NetworkAdapter);

            NetworkUploadContent.Children.Add(NetworkUploadText);
            NetworkUploadContent.Children.Add(NetworkUpload);
            NetworkUploadContent.Children.Add(NetworkUploadType);

            NetworkDownloadContent.Children.Add(NetworkDownloadText);
            NetworkDownloadContent.Children.Add(NetworkDownload);
            NetworkDownloadContent.Children.Add(NetworkDownloadType);

            NetworkPingContent.Children.Add(NetworkPingText);
            NetworkPingContent.Children.Add(NetworkPing);
            NetworkPingContent.Children.Add(NetworkPingType);

            NetworkContent.Children.Add(NetworkAdapterContent);
            NetworkContent.Children.Add(NetworkUploadContent);
            NetworkContent.Children.Add(NetworkDownloadContent);
            NetworkContent.Children.Add(NetworkPingContent);

            Network.FooterCard = NetworkContent;

            Contents.Add(Network);

            TextBlock LaptopArea = new()
            {
                Foreground = SSRER.GetResource<Brush>("TextFillColorPrimaryBrush"),
                Margin = new Thickness(0, 10, 0, 0),
                FontWeight = FontWeights.Bold,
                Text = "Dizüstü"
            };

            Contents.Add(LaptopArea);

            SPVCEC Battery = new()
            {
                Margin = new Thickness(0, 10, 0, 0)
            };

            Battery.Title.Text = "Pil Gücü";
            Battery.LeftIcon.Symbol = BatterySymbol(SMMM.BatteryUsage);
            Battery.Description.Text = "Dizüstü bilgisayar pil gücünde çalışırken duvar kağıdına ne olacağı.";

            ComboBox BatteryPerformance = new();

            BatteryPerformance.SelectionChanged += (s, e) => BatteryPerformanceSelected(BatteryPerformance.SelectedIndex);

            foreach (SSDEPT Type in Enum.GetValues(typeof(SSDEPT)))
            {
                BatteryPerformance.Items.Add(Type);
            }

            BatteryPerformance.SelectedIndex = (int)SPMM.BatteryPerformance;

            Battery.HeaderFrame = BatteryPerformance;

            StackPanel BatteryContent = new()
            {
                Orientation = Orientation.Horizontal
            };

            TextBlock BatteryUsageText = new()
            {
                Foreground = SSRER.GetResource<Brush>("TextFillColorPrimaryBrush"),
                VerticalAlignment = VerticalAlignment.Center,
                Margin = new Thickness(0, 0, 10, 0),
                FontWeight = FontWeights.SemiBold,
                Text = "Pil Durumu (%):"
            };

            NumberBox BatteryUsage = new()
            {
                Margin = new Thickness(0, 0, 10, 0),
                ClearButtonEnabled = false,
                Value = SMMM.BatteryUsage,
                MaxDecimalPlaces = 0,
                Maximum = 100,
                MaxLength = 3,
                Minimum = 0
            };

            BatteryUsage.ValueChanged += (s, e) => BatteryUsageChanged(Battery, BatteryUsage.Value);

            BatteryContent.Children.Add(BatteryUsageText);
            BatteryContent.Children.Add(BatteryUsage);

            Battery.FooterCard = BatteryContent;

            Contents.Add(Battery);

            SPVCEC Saver = new()
            {
                Margin = new Thickness(0, 10, 0, 0),
                Expandable = false
            };

            Saver.Title.Text = "Pil Tasarrufu";
            Saver.LeftIcon.Symbol = SymbolRegular.BatterySaver24;
            Saver.Description.Text = "Dizüstü bilgisayar pil tasarrufu modundayken duvar kağıdına ne olacağı.";

            ComboBox SaverPerformance = new();

            SaverPerformance.SelectionChanged += (s, e) => SaverPerformanceSelected(SaverPerformance.SelectedIndex);

            foreach (SSDEPT Type in Enum.GetValues(typeof(SSDEPT)))
            {
                SaverPerformance.Items.Add(Type);
            }

            SaverPerformance.SelectedIndex = (int)SPMM.SaverPerformance;

            Saver.HeaderFrame = SaverPerformance;

            Contents.Add(Saver);

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

        private SymbolRegular BatterySymbol(int Value)
        {
            if (Value <= 0)
            {
                return SymbolRegular.Battery024;
            }
            else if (Value <= 10)
            {
                return SymbolRegular.Battery124;
            }
            else if (Value <= 20)
            {
                return SymbolRegular.Battery224;
            }
            else if (Value <= 30)
            {
                return SymbolRegular.Battery324;
            }
            else if (Value <= 40)
            {
                return SymbolRegular.Battery424;
            }
            else if (Value <= 50)
            {
                return SymbolRegular.Battery524;
            }
            else if (Value <= 60)
            {
                return SymbolRegular.Battery624;
            }
            else if (Value <= 70)
            {
                return SymbolRegular.Battery724;
            }
            else if (Value <= 80)
            {
                return SymbolRegular.Battery824;
            }
            else if (Value <= 90)
            {
                return SymbolRegular.Battery924;
            }
            else
            {
                return SymbolRegular.Battery1024;
            }
        }

        private void CpuUsageChanged(double? Value)
        {
            int NewValue = Convert.ToInt32(Value);

            if (NewValue != SMMM.CpuUsage)
            {
                SMMI.BackgroundogSettingManager.SetSetting(SMC.CpuUsage, NewValue);
            }
        }

        private void CpuPerformanceSelected(int Index)
        {
            if (Index != (int)SPMM.CpuPerformance)
            {
                SSDEPT Type = (SSDEPT)Index;

                SMMI.BackgroundogSettingManager.SetSetting(SMC.CpuPerformance, Type);
            }
        }

        private void MemoryUsageChanged(double? Value)
        {
            int NewValue = Convert.ToInt32(Value);

            if (NewValue != SMMM.MemoryUsage)
            {
                SMMI.BackgroundogSettingManager.SetSetting(SMC.MemoryUsage, NewValue);
            }
        }

        private void NetworkPingChanged(double? Value)
        {
            int NewValue = Convert.ToInt32(Value);

            if (NewValue != SMMM.PingValue)
            {
                SMMI.BackgroundogSettingManager.SetSetting(SMC.PingValue, NewValue);
            }
        }

        private void NetworkUploadChanged(double? Value)
        {
            int NewValue = Convert.ToInt32(Value);

            if (NewValue != SMMM.UploadValue)
            {
                SMMI.BackgroundogSettingManager.SetSetting(SMC.UploadValue, NewValue);
            }
        }

        private void SaverPerformanceSelected(int Index)
        {
            if (Index != (int)SPMM.SaverPerformance)
            {
                SSDEPT Type = (SSDEPT)Index;

                SMMI.BackgroundogSettingManager.SetSetting(SMC.SaverPerformance, Type);
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

        private void MemoryPerformanceSelected(int Index)
        {
            if (Index != (int)SPMM.MemoryPerformance)
            {
                SSDEPT Type = (SSDEPT)Index;

                SMMI.BackgroundogSettingManager.SetSetting(SMC.MemoryPerformance, Type);
            }
        }

        private void NetworkPingTypeSelected(string Value)
        {
            if (Value != SMMM.PingType)
            {
                SMMI.BackgroundogSettingManager.SetSetting(SMC.PingType, Value);
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

        private void BatteryPerformanceSelected(int Index)
        {
            if (Index != (int)SPMM.BatteryPerformance)
            {
                SSDEPT Type = (SSDEPT)Index;

                SMMI.BackgroundogSettingManager.SetSetting(SMC.BatteryPerformance, Type);
            }
        }

        private void NetworkDownloadChanged(double? Value)
        {
            int NewValue = Convert.ToInt32(Value);

            if (NewValue != SMMM.DownloadValue)
            {
                SMMI.BackgroundogSettingManager.SetSetting(SMC.DownloadValue, NewValue);
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

        private void CounterStateChecked(SPVCEC Battery, bool State)
        {
            Battery.IsExpand = !State;
            Battery.Expandable = !State;
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

        private void BatteryUsageChanged(SPVCEC Battery, double? Value)
        {
            int NewValue = Convert.ToInt32(Value);

            if (NewValue != SMMM.BatteryUsage)
            {
                Battery.LeftIcon.Symbol = BatterySymbol(NewValue);

                SMMI.BackgroundogSettingManager.SetSetting(SMC.BatteryUsage, NewValue);
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