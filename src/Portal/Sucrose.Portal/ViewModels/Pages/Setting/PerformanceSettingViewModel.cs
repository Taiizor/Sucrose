using CommunityToolkit.Mvvm.ComponentModel;
using System.Windows;
using System.Windows.Controls;
using System.Windows.Media;
using Wpf.Ui.Controls;
using SEST = Skylark.Enum.StorageType;
using SMMB = Sucrose.Manager.Manage.Backgroundog;
using SMMCB = Sucrose.Memory.Manage.Constant.Backgroundog;
using SMMI = Sucrose.Manager.Manage.Internal;
using SMMRA = Sucrose.Memory.Manage.Readonly.App;
using SMMRG = Sucrose.Memory.Manage.Readonly.General;
using SMMS = Sucrose.Manager.Manage.System;
using SPVCEC = Sucrose.Portal.Views.Controls.ExpanderCard;
using SRER = Sucrose.Resources.Extension.Resources;
using SSDECDT = Sucrose.Shared.Dependency.Enum.CommandType;
using SSDECNT = Sucrose.Shared.Dependency.Enum.CommunicationType;
using SSDEPPT = Sucrose.Shared.Dependency.Enum.PausePerformanceType;
using SSDEPT = Sucrose.Shared.Dependency.Enum.PerformanceType;
using SSDMMB = Sucrose.Shared.Dependency.Manage.Manager.Backgroundog;
using SSDSHS = Sucrose.Shared.Dependency.Struct.HostStruct;
using SSSHN = Sucrose.Shared.Space.Helper.Network;
using SSSHP = Sucrose.Shared.Space.Helper.Processor;
using SSSMI = Sucrose.Shared.Space.Manage.Internal;
using TextBlock = System.Windows.Controls.TextBlock;

namespace Sucrose.Portal.ViewModels.Pages
{
    public partial class PerformanceSettingViewModel : ViewModel, IDisposable
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
                Foreground = SRER.GetResource<Brush>("TextFillColorPrimaryBrush"),
                Text = SRER.GetValue("Portal", "Area", "AppearanceBehavior"),
                Margin = new Thickness(0, 0, 0, 0),
                FontWeight = FontWeights.Bold
            };

            Contents.Add(AppearanceBehaviorArea);

            SPVCEC Counter = new()
            {
                Expandable = !SMMB.PerformanceCounter,
                IsExpand = !SMMB.PerformanceCounter,
                Margin = new Thickness(0, 10, 0, 0)
            };

            Counter.LeftIcon.Symbol = SymbolRegular.ShiftsActivity24;
            Counter.Title.Text = SRER.GetValue("Portal", "PerformanceSettingPage", "Counter");
            Counter.Description.Text = SRER.GetValue("Portal", "PerformanceSettingPage", "Counter", "Description");

            ToggleSwitch CounterState = new()
            {
                IsChecked = SMMB.PerformanceCounter
            };

            CounterState.Checked += (s, e) => CounterStateChecked(Counter, true);
            CounterState.Unchecked += (s, e) => CounterStateChecked(Counter, false);

            Counter.HeaderFrame = CounterState;

            TextBlock CounterHint = new()
            {
                Text = SRER.GetValue("Portal", "PerformanceSettingPage", "Counter", "CounterHint"),
                Foreground = SRER.GetResource<Brush>("TextFillColorSecondaryBrush"),
                HorizontalAlignment = HorizontalAlignment.Left,
                TextWrapping = TextWrapping.WrapWithOverflow,
                TextAlignment = TextAlignment.Left,
                Margin = new Thickness(0, 0, 0, 0),
                FontWeight = FontWeights.SemiBold
            };

            Counter.FooterCard = CounterHint;

            Contents.Add(Counter);

            SPVCEC Pause = new()
            {
                Expandable = false,
                Margin = new Thickness(0, 10, 0, 0)
            };

            Pause.LeftIcon.Symbol = SymbolRegular.PauseSettings20;
            Pause.Title.Text = SRER.GetValue("Portal", "PerformanceSettingPage", "Pause");
            Pause.Description.Text = SRER.GetValue("Portal", "PerformanceSettingPage", "Pause", "Description");

            ComboBox PausePerformanceType = new();

            PausePerformanceType.SelectionChanged += (s, e) => PausePerformanceTypeSelected(PausePerformanceType.SelectedIndex);

            foreach (SSDEPPT Type in Enum.GetValues(typeof(SSDEPPT)))
            {
                PausePerformanceType.Items.Add(SRER.GetValue("Portal", "Enum", "PausePerformanceType", $"{Type}"));
            }

            PausePerformanceType.SelectedIndex = (int)SSDMMB.PausePerformanceType;

            Pause.HeaderFrame = PausePerformanceType;

            Contents.Add(Pause);

            SPVCEC Communication = new()
            {
                Expandable = false,
                Margin = new Thickness(0, 10, 0, 0)
            };

            Communication.LeftIcon.Symbol = SymbolRegular.Communication24;
            Communication.Title.Text = SRER.GetValue("Portal", "PerformanceSettingPage", "Communication");
            Communication.Description.Text = SRER.GetValue("Portal", "PerformanceSettingPage", "Communication", "Description");

            ComboBox CommunicationType = new();

            CommunicationType.SelectionChanged += (s, e) => CommunicationTypeSelected(CommunicationType.SelectedIndex);

            foreach (SSDECNT Type in Enum.GetValues(typeof(SSDECNT)))
            {
                CommunicationType.Items.Add(SRER.GetValue("Portal", "Enum", "CommunicationType", $"{Type}"));
            }

            CommunicationType.SelectedIndex = (int)SSDMMB.CommunicationType;

            Communication.HeaderFrame = CommunicationType;

            Contents.Add(Communication);

            TextBlock SystemResourcesArea = new()
            {
                Foreground = SRER.GetResource<Brush>("TextFillColorPrimaryBrush"),
                Text = SRER.GetValue("Portal", "Area", "SystemResources"),
                Margin = new Thickness(0, 10, 0, 0),
                FontWeight = FontWeights.Bold
            };

            Contents.Add(SystemResourcesArea);

            SPVCEC Cpu = new()
            {
                Margin = new Thickness(0, 10, 0, 0)
            };

            Cpu.LeftIcon.Symbol = SymbolRegular.DeveloperBoardLightning20;
            Cpu.Title.Text = SRER.GetValue("Portal", "PerformanceSettingPage", "Cpu");
            Cpu.Description.Text = SRER.GetValue("Portal", "PerformanceSettingPage", "Cpu", "Description");

            ComboBox CpuPerformance = new();

            CpuPerformance.SelectionChanged += (s, e) => CpuPerformanceSelected(CpuPerformance.SelectedIndex);

            foreach (SSDEPT Type in Enum.GetValues(typeof(SSDEPT)))
            {
                CpuPerformance.Items.Add(SRER.GetValue("Portal", "Enum", "PerformanceType", $"{Type}"));
            }

            CpuPerformance.SelectedIndex = (int)SSDMMB.CpuPerformance;

            Cpu.HeaderFrame = CpuPerformance;

            StackPanel CpuContent = new()
            {
                Orientation = Orientation.Horizontal
            };

            TextBlock CpuUsageText = new()
            {
                Text = SRER.GetValue("Portal", "PerformanceSettingPage", "Cpu", "CpuUsage"),
                Foreground = SRER.GetResource<Brush>("TextFillColorPrimaryBrush"),
                VerticalAlignment = VerticalAlignment.Center,
                Margin = new Thickness(0, 0, 10, 0),
                FontWeight = FontWeights.SemiBold
            };

            NumberBox CpuUsage = new()
            {
                Icon = new SymbolIcon(SymbolRegular.ArrowTrendingLines24),
                IconPlacement = ElementPlacement.Left,
                Margin = new Thickness(0, 0, 10, 0),
                ClearButtonEnabled = false,
                Value = SMMB.CpuUsage,
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

            SPVCEC Gpu = new()
            {
                Margin = new Thickness(0, 10, 0, 0)
            };

            Gpu.LeftIcon.Symbol = SymbolRegular.VideoPerson24;
            Gpu.Title.Text = SRER.GetValue("Portal", "PerformanceSettingPage", "Gpu");
            Gpu.Description.Text = SRER.GetValue("Portal", "PerformanceSettingPage", "Gpu", "Description");

            ComboBox GpuPerformance = new();

            GpuPerformance.SelectionChanged += (s, e) => GpuPerformanceSelected(GpuPerformance.SelectedIndex);

            foreach (SSDEPT Type in Enum.GetValues(typeof(SSDEPT)))
            {
                GpuPerformance.Items.Add(SRER.GetValue("Portal", "Enum", "PerformanceType", $"{Type}"));
            }

            GpuPerformance.SelectedIndex = (int)SSDMMB.GpuPerformance;

            Gpu.HeaderFrame = GpuPerformance;

            StackPanel GpuContent = new();

            StackPanel GpuAdapterContent = new()
            {
                Orientation = Orientation.Horizontal
            };

            TextBlock GpuAdapterText = new()
            {
                Text = SRER.GetValue("Portal", "PerformanceSettingPage", "Gpu", "GpuAdapter"),
                Foreground = SRER.GetResource<Brush>("TextFillColorPrimaryBrush"),
                VerticalAlignment = VerticalAlignment.Center,
                Margin = new Thickness(0, 0, 10, 0),
                FontWeight = FontWeights.SemiBold
            };

            ComboBox GpuAdapter = new()
            {
                MaxDropDownHeight = 200,
                MaxWidth = 700
            };

            DynamicScrollViewer.SetVerticalScrollBarVisibility(GpuAdapter, ScrollBarVisibility.Auto);

            if (SMMS.GraphicInterfaces.Any())
            {
                GpuAdapter.SelectionChanged += (s, e) => GpuAdapterSelected($"{GpuAdapter.SelectedValue}");

                foreach (string Interface in SMMS.GraphicInterfaces)
                {
                    GpuAdapter.Items.Add(Interface);
                }

                string SelectedAdapter = SMMB.GraphicAdapter;

                if (string.IsNullOrEmpty(SelectedAdapter))
                {
                    GpuAdapter.SelectedIndex = 0;
                }
                else
                {
                    GpuAdapter.SelectedValue = SelectedAdapter;
                }
            }
            else
            {
                GpuAdapter.Items.Add(new ComboBoxItem()
                {
                    Content = SRER.GetValue("Portal", "PerformanceSettingPage", "Gpu", "GpuAdapter", "Empty"),
                    IsSelected = true
                });
            }

            StackPanel GpuUsageContent = new()
            {
                Orientation = Orientation.Horizontal,
                Margin = new Thickness(0, 10, 0, 0)
            };

            TextBlock GpuUsageText = new()
            {
                Text = SRER.GetValue("Portal", "PerformanceSettingPage", "Gpu", "GpuUsage"),
                Foreground = SRER.GetResource<Brush>("TextFillColorPrimaryBrush"),
                VerticalAlignment = VerticalAlignment.Center,
                Margin = new Thickness(0, 0, 10, 0),
                FontWeight = FontWeights.SemiBold
            };

            NumberBox GpuUsage = new()
            {
                Icon = new SymbolIcon(SymbolRegular.ArrowTrendingSparkle24),
                IconPlacement = ElementPlacement.Left,
                Margin = new Thickness(0, 0, 10, 0),
                ClearButtonEnabled = false,
                Value = SMMB.GpuUsage,
                MaxDecimalPlaces = 0,
                Maximum = 100,
                MaxLength = 3,
                Minimum = 0
            };

            GpuUsage.ValueChanged += (s, e) => GpuUsageChanged(GpuUsage.Value);

            GpuAdapterContent.Children.Add(GpuAdapterText);
            GpuAdapterContent.Children.Add(GpuAdapter);

            GpuUsageContent.Children.Add(GpuUsageText);
            GpuUsageContent.Children.Add(GpuUsage);

            GpuContent.Children.Add(GpuAdapterContent);
            GpuContent.Children.Add(GpuUsageContent);

            Gpu.FooterCard = GpuContent;

            Contents.Add(Gpu);

            SPVCEC Memory = new()
            {
                Margin = new Thickness(0, 10, 0, 0)
            };

            Memory.LeftIcon.Symbol = SymbolRegular.Memory16;
            Memory.Title.Text = SRER.GetValue("Portal", "PerformanceSettingPage", "Memory");
            Memory.Description.Text = SRER.GetValue("Portal", "PerformanceSettingPage", "Memory", "Description");

            ComboBox MemoryPerformance = new();

            MemoryPerformance.SelectionChanged += (s, e) => MemoryPerformanceSelected(MemoryPerformance.SelectedIndex);

            foreach (SSDEPT Type in Enum.GetValues(typeof(SSDEPT)))
            {
                MemoryPerformance.Items.Add(SRER.GetValue("Portal", "Enum", "PerformanceType", $"{Type}"));
            }

            MemoryPerformance.SelectedIndex = (int)SSDMMB.MemoryPerformance;

            Memory.HeaderFrame = MemoryPerformance;

            StackPanel MemoryContent = new()
            {
                Orientation = Orientation.Horizontal
            };

            TextBlock MemoryUsageText = new()
            {
                Text = SRER.GetValue("Portal", "PerformanceSettingPage", "Memory", "MemoryUsage"),
                Foreground = SRER.GetResource<Brush>("TextFillColorPrimaryBrush"),
                VerticalAlignment = VerticalAlignment.Center,
                Margin = new Thickness(0, 0, 10, 0),
                FontWeight = FontWeights.SemiBold
            };

            NumberBox MemoryUsage = new()
            {
                Icon = new SymbolIcon(SymbolRegular.DataUsage24),
                IconPlacement = ElementPlacement.Left,
                Margin = new Thickness(0, 0, 10, 0),
                ClearButtonEnabled = false,
                Value = SMMB.MemoryUsage,
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

            Network.LeftIcon.Symbol = SymbolRegular.NetworkCheck24;
            Network.Title.Text = SRER.GetValue("Portal", "PerformanceSettingPage", "Network");
            Network.Description.Text = SRER.GetValue("Portal", "PerformanceSettingPage", "Network", "Description");

            ComboBox NetworkPerformance = new();

            NetworkPerformance.SelectionChanged += (s, e) => NetworkPerformanceSelected(NetworkPerformance.SelectedIndex);

            foreach (SSDEPT Type in Enum.GetValues(typeof(SSDEPT)))
            {
                NetworkPerformance.Items.Add(SRER.GetValue("Portal", "Enum", "PerformanceType", $"{Type}"));
            }

            NetworkPerformance.SelectedIndex = (int)SSDMMB.NetworkPerformance;

            Network.HeaderFrame = NetworkPerformance;

            StackPanel NetworkContent = new();

            StackPanel NetworkAdapterContent = new()
            {
                Orientation = Orientation.Horizontal
            };

            TextBlock NetworkAdapterText = new()
            {
                Text = SRER.GetValue("Portal", "PerformanceSettingPage", "Network", "NetworkAdapter"),
                Foreground = SRER.GetResource<Brush>("TextFillColorPrimaryBrush"),
                VerticalAlignment = VerticalAlignment.Center,
                Margin = new Thickness(0, 0, 10, 0),
                FontWeight = FontWeights.SemiBold
            };

            ComboBox NetworkAdapter = new()
            {
                MaxDropDownHeight = 200,
                MaxWidth = 700
            };

            DynamicScrollViewer.SetVerticalScrollBarVisibility(NetworkAdapter, ScrollBarVisibility.Auto);

            if (SMMS.NetworkInterfaces.Any())
            {
                NetworkAdapter.SelectionChanged += (s, e) => NetworkAdapterSelected($"{NetworkAdapter.SelectedValue}");

                foreach (string Interface in SMMS.NetworkInterfaces)
                {
                    NetworkAdapter.Items.Add(Interface);
                }

                string SelectedAdapter = SMMB.NetworkAdapter;

                if (string.IsNullOrEmpty(SelectedAdapter))
                {
                    NetworkAdapter.SelectedIndex = 0;
                }
                else
                {
                    NetworkAdapter.SelectedValue = SelectedAdapter;
                }
            }
            else
            {
                NetworkAdapter.Items.Add(new ComboBoxItem()
                {
                    Content = SRER.GetValue("Portal", "PerformanceSettingPage", "Network", "NetworkAdapter", "Empty"),
                    IsSelected = true
                });
            }

            StackPanel NetworkUploadContent = new()
            {
                Orientation = Orientation.Horizontal,
                Margin = new Thickness(0, 10, 0, 0)
            };

            TextBlock NetworkUploadText = new()
            {
                Text = SRER.GetValue("Portal", "PerformanceSettingPage", "Network", "NetworkUpload"),
                Foreground = SRER.GetResource<Brush>("TextFillColorPrimaryBrush"),
                VerticalAlignment = VerticalAlignment.Center,
                Margin = new Thickness(0, 0, 10, 0),
                FontWeight = FontWeights.SemiBold
            };

            NumberBox NetworkUpload = new()
            {
                Icon = new SymbolIcon(SymbolRegular.ArrowUpload24),
                IconPlacement = ElementPlacement.Left,
                Margin = new Thickness(0, 0, 10, 0),
                ClearButtonEnabled = false,
                Value = SMMB.UploadValue,
                MaxDecimalPlaces = 0,
                Maximum = 99999999,
                MaxLength = 8,
                Minimum = 0
            };

            NetworkUpload.ValueChanged += (s, e) => NetworkUploadChanged(NetworkUpload.Value);

            ComboBox NetworkUploadType = new()
            {
                MaxDropDownHeight = 200
            };

            DynamicScrollViewer.SetVerticalScrollBarVisibility(NetworkUploadType, ScrollBarVisibility.Auto);

            NetworkUploadType.SelectionChanged += (s, e) => NetworkUploadTypeSelected(NetworkUploadType.SelectedIndex);

            foreach (SEST Type in Enum.GetValues(typeof(SEST)))
            {
                NetworkUploadType.Items.Add(Type);
            }

            NetworkUploadType.SelectedIndex = (int)SMMB.UploadType;

            StackPanel NetworkDownloadContent = new()
            {
                Orientation = Orientation.Horizontal,
                Margin = new Thickness(0, 10, 0, 0)
            };

            TextBlock NetworkDownloadText = new()
            {
                Text = SRER.GetValue("Portal", "PerformanceSettingPage", "Network", "NetworkDownload"),
                Foreground = SRER.GetResource<Brush>("TextFillColorPrimaryBrush"),
                VerticalAlignment = VerticalAlignment.Center,
                Margin = new Thickness(0, 0, 10, 0),
                FontWeight = FontWeights.SemiBold
            };

            NumberBox NetworkDownload = new()
            {
                Icon = new SymbolIcon(SymbolRegular.ArrowDownload24),
                IconPlacement = ElementPlacement.Left,
                Margin = new Thickness(0, 0, 10, 0),
                ClearButtonEnabled = false,
                Value = SMMB.DownloadValue,
                MaxDecimalPlaces = 0,
                Maximum = 99999999,
                MaxLength = 8,
                Minimum = 0
            };

            NetworkDownload.ValueChanged += (s, e) => NetworkDownloadChanged(NetworkDownload.Value);

            ComboBox NetworkDownloadType = new()
            {
                MaxDropDownHeight = 200
            };

            DynamicScrollViewer.SetVerticalScrollBarVisibility(NetworkDownloadType, ScrollBarVisibility.Auto);

            NetworkDownloadType.SelectionChanged += (s, e) => NetworkDownloadTypeSelected(NetworkDownloadType.SelectedIndex);

            foreach (SEST Type in Enum.GetValues(typeof(SEST)))
            {
                NetworkDownloadType.Items.Add(Type);
            }

            NetworkDownloadType.SelectedIndex = (int)SMMB.DownloadType;

            StackPanel NetworkPingContent = new()
            {
                Orientation = Orientation.Horizontal,
                Margin = new Thickness(0, 10, 0, 0)
            };

            TextBlock NetworkPingText = new()
            {
                Text = SRER.GetValue("Portal", "PerformanceSettingPage", "Network", "NetworkPing"),
                Foreground = SRER.GetResource<Brush>("TextFillColorPrimaryBrush"),
                VerticalAlignment = VerticalAlignment.Center,
                Margin = new Thickness(0, 0, 10, 0),
                FontWeight = FontWeights.SemiBold
            };

            NumberBox NetworkPing = new()
            {
                Icon = new SymbolIcon(SymbolRegular.HeartPulse24),
                IconPlacement = ElementPlacement.Left,
                Margin = new Thickness(0, 0, 10, 0),
                ClearButtonEnabled = false,
                Value = SMMB.PingValue,
                MaxDecimalPlaces = 0,
                Maximum = 1000,
                MaxLength = 4,
                Minimum = 0
            };

            NetworkPing.ValueChanged += (s, e) => NetworkPingChanged(NetworkPing.Value);

            ComboBox NetworkPingType = new()
            {
                MaxDropDownHeight = 200
            };

            DynamicScrollViewer.SetVerticalScrollBarVisibility(NetworkPingType, ScrollBarVisibility.Auto);

            NetworkPingType.SelectionChanged += (s, e) => NetworkPingTypeSelected($"{NetworkPingType.SelectedValue}");

            foreach (SSDSHS Host in SSSHN.GetHost())
            {
                NetworkPingType.Items.Add(Host.Name);
            }

            NetworkPingType.SelectedValue = SMMB.PingType;

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
                Foreground = SRER.GetResource<Brush>("TextFillColorPrimaryBrush"),
                Text = SRER.GetValue("Portal", "Area", "Laptop"),
                Margin = new Thickness(0, 10, 0, 0),
                FontWeight = FontWeights.Bold
            };

            Contents.Add(LaptopArea);

            SPVCEC Battery = new()
            {
                Margin = new Thickness(0, 10, 0, 0)
            };

            Battery.LeftIcon.Symbol = BatterySymbol(SMMB.BatteryUsage);
            Battery.Title.Text = SRER.GetValue("Portal", "PerformanceSettingPage", "Battery");
            Battery.Description.Text = SRER.GetValue("Portal", "PerformanceSettingPage", "Battery", "Description");

            ComboBox BatteryPerformance = new();

            BatteryPerformance.SelectionChanged += (s, e) => BatteryPerformanceSelected(BatteryPerformance.SelectedIndex);

            foreach (SSDEPT Type in Enum.GetValues(typeof(SSDEPT)))
            {
                BatteryPerformance.Items.Add(SRER.GetValue("Portal", "Enum", "PerformanceType", $"{Type}"));
            }

            BatteryPerformance.SelectedIndex = (int)SSDMMB.BatteryPerformance;

            Battery.HeaderFrame = BatteryPerformance;

            StackPanel BatteryContent = new()
            {
                Orientation = Orientation.Horizontal
            };

            TextBlock BatteryUsageText = new()
            {
                Text = SRER.GetValue("Portal", "PerformanceSettingPage", "Battery", "BatteryUsage"),
                Foreground = SRER.GetResource<Brush>("TextFillColorPrimaryBrush"),
                VerticalAlignment = VerticalAlignment.Center,
                Margin = new Thickness(0, 0, 10, 0),
                FontWeight = FontWeights.SemiBold
            };

            NumberBox BatteryUsage = new()
            {
                Icon = new SymbolIcon(SymbolRegular.BatteryWarning24),
                IconPlacement = ElementPlacement.Left,
                Margin = new Thickness(0, 0, 10, 0),
                ClearButtonEnabled = false,
                Value = SMMB.BatteryUsage,
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

            Saver.LeftIcon.Symbol = SymbolRegular.BatterySaver24;
            Saver.Title.Text = SRER.GetValue("Portal", "PerformanceSettingPage", "Saver");
            Saver.Description.Text = SRER.GetValue("Portal", "PerformanceSettingPage", "Saver", "Description");

            ComboBox SaverPerformance = new();

            SaverPerformance.SelectionChanged += (s, e) => SaverPerformanceSelected(SaverPerformance.SelectedIndex);

            foreach (SSDEPT Type in Enum.GetValues(typeof(SSDEPT)))
            {
                SaverPerformance.Items.Add(SRER.GetValue("Portal", "Enum", "PerformanceType", $"{Type}"));
            }

            SaverPerformance.SelectedIndex = (int)SSDMMB.SaverPerformance;

            Saver.HeaderFrame = SaverPerformance;

            Contents.Add(Saver);

            TextBlock SystemArea = new()
            {
                Foreground = SRER.GetResource<Brush>("TextFillColorPrimaryBrush"),
                Text = SRER.GetValue("Portal", "Area", "System"),
                Margin = new Thickness(0, 10, 0, 0),
                FontWeight = FontWeights.Bold
            };

            Contents.Add(SystemArea);

            SPVCEC Virtual = new()
            {
                Margin = new Thickness(0, 10, 0, 0),
                Expandable = false
            };

            Virtual.LeftIcon.Symbol = SymbolRegular.DesktopCheckmark24;
            Virtual.Title.Text = SRER.GetValue("Portal", "PerformanceSettingPage", "Virtual");
            Virtual.Description.Text = SRER.GetValue("Portal", "PerformanceSettingPage", "Virtual", "Description");

            ComboBox VirtualPerformance = new();

            VirtualPerformance.SelectionChanged += (s, e) => VirtualPerformanceSelected(VirtualPerformance.SelectedIndex);

            foreach (SSDEPT Type in Enum.GetValues(typeof(SSDEPT)))
            {
                VirtualPerformance.Items.Add(SRER.GetValue("Portal", "Enum", "PerformanceType", $"{Type}"));
            }

            VirtualPerformance.SelectedIndex = (int)SSDMMB.VirtualPerformance;

            Virtual.HeaderFrame = VirtualPerformance;

            Contents.Add(Virtual);

            SPVCEC Remote = new()
            {
                Margin = new Thickness(0, 10, 0, 0),
                Expandable = false
            };

            Remote.LeftIcon.Symbol = SymbolRegular.Remote20;
            Remote.Title.Text = SRER.GetValue("Portal", "PerformanceSettingPage", "Remote");
            Remote.Description.Text = SRER.GetValue("Portal", "PerformanceSettingPage", "Remote", "Description");

            ComboBox RemotePerformance = new();

            RemotePerformance.SelectionChanged += (s, e) => RemotePerformanceSelected(RemotePerformance.SelectedIndex);

            foreach (SSDEPT Type in Enum.GetValues(typeof(SSDEPT)))
            {
                RemotePerformance.Items.Add(SRER.GetValue("Portal", "Enum", "PerformanceType", $"{Type}"));
            }

            RemotePerformance.SelectedIndex = (int)SSDMMB.RemotePerformance;

            Remote.HeaderFrame = RemotePerformance;

            Contents.Add(Remote);

            SPVCEC Fullscreen = new()
            {
                Margin = new Thickness(0, 10, 0, 0),
                Expandable = false
            };

            Fullscreen.LeftIcon.Symbol = SymbolRegular.FullScreenMaximize24;
            Fullscreen.Title.Text = SRER.GetValue("Portal", "PerformanceSettingPage", "Fullscreen");
            Fullscreen.Description.Text = SRER.GetValue("Portal", "PerformanceSettingPage", "Fullscreen", "Description");

            ComboBox FullscreenPerformance = new();

            FullscreenPerformance.SelectionChanged += (s, e) => FullscreenPerformanceSelected(FullscreenPerformance.SelectedIndex);

            foreach (SSDEPT Type in Enum.GetValues(typeof(SSDEPT)))
            {
                FullscreenPerformance.Items.Add(SRER.GetValue("Portal", "Enum", "PerformanceType", $"{Type}"));
            }

            FullscreenPerformance.SelectedIndex = (int)SSDMMB.FullscreenPerformance;

            Fullscreen.HeaderFrame = FullscreenPerformance;

            Contents.Add(Fullscreen);

            SPVCEC Focus = new()
            {
                Margin = new Thickness(0, 10, 0, 0),
                Expandable = false
            };

            Focus.LeftIcon.Symbol = SymbolRegular.CursorHover24;
            Focus.Title.Text = SRER.GetValue("Portal", "PerformanceSettingPage", "Focus");
            Focus.Description.Text = SRER.GetValue("Portal", "PerformanceSettingPage", "Focus", "Description");

            ComboBox FocusPerformance = new();

            FocusPerformance.SelectionChanged += (s, e) => FocusPerformanceSelected(FocusPerformance.SelectedIndex);

            foreach (SSDEPT Type in Enum.GetValues(typeof(SSDEPT)))
            {
                FocusPerformance.Items.Add(SRER.GetValue("Portal", "Enum", "PerformanceType", $"{Type}"));
            }

            FocusPerformance.SelectedIndex = (int)SSDMMB.FocusPerformance;

            Focus.HeaderFrame = FocusPerformance;

            Contents.Add(Focus);

            _isInitialized = true;
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

            if (NewValue != SMMB.CpuUsage)
            {
                SMMI.BackgroundogSettingManager.SetSetting(SMMCB.CpuUsage, NewValue);
            }
        }

        private void GpuUsageChanged(double? Value)
        {
            int NewValue = Convert.ToInt32(Value);

            if (NewValue != SMMB.GpuUsage)
            {
                SMMI.BackgroundogSettingManager.SetSetting(SMMCB.GpuUsage, NewValue);
            }
        }

        private void GpuAdapterSelected(string Value)
        {
            if (Value != SMMB.GraphicAdapter)
            {
                SMMI.BackgroundogSettingManager.SetSetting(SMMCB.GraphicAdapter, Value);
            }
        }

        private void CpuPerformanceSelected(int Index)
        {
            if (Index != (int)SSDMMB.CpuPerformance)
            {
                SSDEPT Type = (SSDEPT)Index;

                SMMI.BackgroundogSettingManager.SetSetting(SMMCB.CpuPerformance, Type);
            }
        }

        private void GpuPerformanceSelected(int Index)
        {
            if (Index != (int)SSDMMB.GpuPerformance)
            {
                SSDEPT Type = (SSDEPT)Index;

                SMMI.BackgroundogSettingManager.SetSetting(SMMCB.GpuPerformance, Type);
            }
        }

        private void MemoryUsageChanged(double? Value)
        {
            int NewValue = Convert.ToInt32(Value);

            if (NewValue != SMMB.MemoryUsage)
            {
                SMMI.BackgroundogSettingManager.SetSetting(SMMCB.MemoryUsage, NewValue);
            }
        }

        private void NetworkPingChanged(double? Value)
        {
            int NewValue = Convert.ToInt32(Value);

            if (NewValue != SMMB.PingValue)
            {
                SMMI.BackgroundogSettingManager.SetSetting(SMMCB.PingValue, NewValue);
            }
        }

        private void NetworkUploadChanged(double? Value)
        {
            int NewValue = Convert.ToInt32(Value);

            if (NewValue != SMMB.UploadValue)
            {
                SMMI.BackgroundogSettingManager.SetSetting(SMMCB.UploadValue, NewValue);
            }
        }

        private void SaverPerformanceSelected(int Index)
        {
            if (Index != (int)SSDMMB.SaverPerformance)
            {
                SSDEPT Type = (SSDEPT)Index;

                SMMI.BackgroundogSettingManager.SetSetting(SMMCB.SaverPerformance, Type);
            }
        }

        private void FocusPerformanceSelected(int Index)
        {
            if (Index != (int)SSDMMB.FocusPerformance)
            {
                SSDEPT Type = (SSDEPT)Index;

                SMMI.BackgroundogSettingManager.SetSetting(SMMCB.FocusPerformance, Type);
            }
        }

        private void CommunicationTypeSelected(int Index)
        {
            if (Index != (int)SSDMMB.CommunicationType)
            {
                SSDECNT Type = (SSDECNT)Index;

                SMMI.BackgroundogSettingManager.SetSetting(SMMCB.CommunicationType, Type);
            }
        }

        private void RemotePerformanceSelected(int Index)
        {
            if (Index != (int)SSDMMB.RemotePerformance)
            {
                SSDEPT Type = (SSDEPT)Index;

                SMMI.BackgroundogSettingManager.SetSetting(SMMCB.RemotePerformance, Type);
            }
        }

        private void NetworkUploadTypeSelected(int Index)
        {
            if (Index != (int)SMMB.UploadType)
            {
                SEST Type = (SEST)Index;

                SMMI.BackgroundogSettingManager.SetSetting(SMMCB.UploadType, Type);
            }
        }

        private void NetworkAdapterSelected(string Value)
        {
            if (Value != SMMB.NetworkAdapter)
            {
                SMMI.BackgroundogSettingManager.SetSetting(SMMCB.NetworkAdapter, Value);
            }
        }

        private void MemoryPerformanceSelected(int Index)
        {
            if (Index != (int)SSDMMB.MemoryPerformance)
            {
                SSDEPT Type = (SSDEPT)Index;

                SMMI.BackgroundogSettingManager.SetSetting(SMMCB.MemoryPerformance, Type);
            }
        }

        private void VirtualPerformanceSelected(int Index)
        {
            if (Index != (int)SSDMMB.VirtualPerformance)
            {
                SSDEPT Type = (SSDEPT)Index;

                SMMI.BackgroundogSettingManager.SetSetting(SMMCB.VirtualPerformance, Type);
            }
        }

        private void NetworkPingTypeSelected(string Value)
        {
            if (Value != SMMB.PingType)
            {
                SMMI.BackgroundogSettingManager.SetSetting(SMMCB.PingType, Value);
            }
        }

        private void NetworkPerformanceSelected(int Index)
        {
            if (Index != (int)SSDMMB.NetworkPerformance)
            {
                SSDEPT Type = (SSDEPT)Index;

                SMMI.BackgroundogSettingManager.SetSetting(SMMCB.NetworkPerformance, Type);
            }
        }

        private void BatteryPerformanceSelected(int Index)
        {
            if (Index != (int)SSDMMB.BatteryPerformance)
            {
                SSDEPT Type = (SSDEPT)Index;

                SMMI.BackgroundogSettingManager.SetSetting(SMMCB.BatteryPerformance, Type);
            }
        }

        private void NetworkDownloadChanged(double? Value)
        {
            int NewValue = Convert.ToInt32(Value);

            if (NewValue != SMMB.DownloadValue)
            {
                SMMI.BackgroundogSettingManager.SetSetting(SMMCB.DownloadValue, NewValue);
            }
        }

        private void NetworkDownloadTypeSelected(int Index)
        {
            if (Index != (int)SMMB.DownloadType)
            {
                SEST Type = (SEST)Index;

                SMMI.BackgroundogSettingManager.SetSetting(SMMCB.DownloadType, Type);
            }
        }

        private void PausePerformanceTypeSelected(int Index)
        {
            if (Index != (int)SSDMMB.PausePerformanceType)
            {
                SSDEPPT Type = (SSDEPPT)Index;

                SMMI.BackgroundogSettingManager.SetSetting(SMMCB.PausePerformanceType, Type);
            }
        }

        private void FullscreenPerformanceSelected(int Index)
        {
            if (Index != (int)SSDMMB.FullscreenPerformance)
            {
                SSDEPT Type = (SSDEPT)Index;

                SMMI.BackgroundogSettingManager.SetSetting(SMMCB.FullscreenPerformance, Type);
            }
        }

        private void CounterStateChecked(SPVCEC Battery, bool State)
        {
            Battery.IsExpand = !State;
            Battery.Expandable = !State;

            SMMI.BackgroundogSettingManager.SetSetting(SMMCB.PerformanceCounter, State);

            if (State)
            {
                SSSHP.Run(SSSMI.Commandog, $"{SMMRG.StartCommand}{SSDECDT.Backgroundog}{SMMRG.ValueSeparator}{SSSMI.Backgroundog}");
            }
            else
            {
                if (SSSHP.Work(SMMRA.Backgroundog))
                {
                    SSSHP.Kill(SMMRA.Backgroundog);
                }
            }
        }

        private void BatteryUsageChanged(SPVCEC Battery, double? Value)
        {
            int NewValue = Convert.ToInt32(Value);

            if (NewValue != SMMB.BatteryUsage)
            {
                Battery.LeftIcon.Symbol = BatterySymbol(NewValue);

                SMMI.BackgroundogSettingManager.SetSetting(SMMCB.BatteryUsage, NewValue);
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