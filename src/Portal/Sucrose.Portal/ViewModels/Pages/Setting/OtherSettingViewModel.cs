using CommunityToolkit.Mvvm.ComponentModel;
using System.Windows;
using System.Windows.Controls;
using System.Windows.Input;
using System.Windows.Media;
using Wpf.Ui.Controls;
using SEST = Skylark.Enum.StorageType;
using SMMB = Sucrose.Manager.Manage.Backgroundog;
using SMMCE = Sucrose.Memory.Manage.Constant.Engine;
using SMMCG = Sucrose.Memory.Manage.Constant.General;
using SMMCH = Sucrose.Memory.Manage.Constant.Hook;
using SMMCO = Sucrose.Memory.Manage.Constant.Objectionable;
using SMMCU = Sucrose.Memory.Manage.Constant.Update;
using SMME = Sucrose.Manager.Manage.Engine;
using SMMG = Sucrose.Manager.Manage.General;
using SMMH = Sucrose.Manager.Manage.Hook;
using SMMI = Sucrose.Manager.Manage.Internal;
using SMMO = Sucrose.Manager.Manage.Objectionable;
using SMMRA = Sucrose.Memory.Manage.Readonly.App;
using SMMRG = Sucrose.Memory.Manage.Readonly.General;
using SMMRU = Sucrose.Memory.Manage.Readonly.Url;
using SMMU = Sucrose.Manager.Manage.Update;
using SPVCEC = Sucrose.Portal.Views.Controls.ExpanderCard;
using SRER = Sucrose.Resources.Extension.Resources;
using SSCEUCT = Sucrose.Shared.Core.Enum.UpdateChannelType;
using SSCEUET = Sucrose.Shared.Core.Enum.UpdateExtensionType;
using SSCMMU = Sucrose.Shared.Core.Manage.Manager.Update;
using SSDECT = Sucrose.Shared.Dependency.Enum.CommandType;
using SSDEUMT = Sucrose.Shared.Dependency.Enum.UpdateModuleType;
using SSDEUST = Sucrose.Shared.Dependency.Enum.UpdateServerType;
using SSDMMU = Sucrose.Shared.Dependency.Manage.Manager.Update;
using SSSHP = Sucrose.Shared.Space.Helper.Processor;
using SSSPMI = Sucrose.Shared.Space.Manage.Internal;
using SSSTMI = Sucrose.Shared.Store.Manage.Internal;
using TextBlock = System.Windows.Controls.TextBlock;
using TextBox = Wpf.Ui.Controls.TextBox;

namespace Sucrose.Portal.ViewModels.Pages
{
    public partial class OtherSettingViewModel : ViewModel, IDisposable
    {
        [ObservableProperty]
        private List<UIElement> _Contents = new();

        private bool _isInitialized;

        public OtherSettingViewModel()
        {
            if (!_isInitialized)
            {
                InitializeViewModel();
            }
        }

        private void InitializeViewModel()
        {
            TextBlock DataArea = new()
            {
                Foreground = SRER.GetResource<Brush>("TextFillColorPrimaryBrush"),
                Text = SRER.GetValue("Portal", "Area", "Data"),
                Margin = new Thickness(0, 0, 0, 0),
                FontWeight = FontWeights.Bold
            };

            Contents.Add(DataArea);

            SPVCEC ReportData = new()
            {
                Margin = new Thickness(0, 10, 0, 0),
                Expandable = false
            };

            ReportData.LeftIcon.Symbol = SymbolRegular.BugArrowCounterclockwise20;
            ReportData.Title.Text = SRER.GetValue("Portal", "OtherSettingPage", "ReportData");
            ReportData.Description.Text = SRER.GetValue("Portal", "OtherSettingPage", "ReportData", "Description");

            ToggleSwitch ReportState = new()
            {
                IsChecked = SMMG.Report
            };

            ReportState.Checked += (s, e) => ReportStateChecked(true);
            ReportState.Unchecked += (s, e) => ReportStateChecked(false);

            ReportData.HeaderFrame = ReportState;

            Contents.Add(ReportData);

            SPVCEC StatisticsData = new()
            {
                Margin = new Thickness(0, 10, 0, 0),
                Expandable = false
            };

            StatisticsData.LeftIcon.Symbol = SymbolRegular.DataHistogram24;
            StatisticsData.Title.Text = SRER.GetValue("Portal", "OtherSettingPage", "StatisticsData");
            StatisticsData.Description.Text = SRER.GetValue("Portal", "OtherSettingPage", "StatisticsData", "Description");

            ToggleSwitch StatisticsState = new()
            {
                IsChecked = SMMG.Statistics
            };

            StatisticsState.Checked += (s, e) => StatisticsStateChecked(true);
            StatisticsState.Unchecked += (s, e) => StatisticsStateChecked(false);

            StatisticsData.HeaderFrame = StatisticsState;

            Contents.Add(StatisticsData);

            TextBlock HookArea = new()
            {
                Foreground = SRER.GetResource<Brush>("TextFillColorPrimaryBrush"),
                Text = SRER.GetValue("Portal", "Area", "Hook"),
                Margin = new Thickness(0, 0, 0, 0),
                FontWeight = FontWeights.Bold
            };

            Contents.Add(HookArea);

            SPVCEC DiscordHook = new()
            {
                Margin = new Thickness(0, 10, 0, 0),
                IsExpand = true
            };

            DiscordHook.LeftIcon.Symbol = SymbolRegular.SquareHintApps24;
            DiscordHook.Title.Text = SRER.GetValue("Portal", "OtherSettingPage", "DiscordHook");
            DiscordHook.Description.Text = SRER.GetValue("Portal", "OtherSettingPage", "DiscordHook", "Description");

            ToggleSwitch DiscordState = new()
            {
                IsChecked = SMMH.DiscordState
            };

            DiscordState.Checked += (s, e) => DiscordStateChecked(true);
            DiscordState.Unchecked += (s, e) => DiscordStateChecked(false);

            DiscordHook.HeaderFrame = DiscordState;

            StackPanel DiscordContent = new();

            StackPanel DiscordRefreshContent = new()
            {
                Orientation = Orientation.Horizontal
            };

            TextBlock DiscordRefreshText = new()
            {
                Text = SRER.GetValue("Portal", "OtherSettingPage", "DiscordHook", "DiscordRefresh"),
                Foreground = SRER.GetResource<Brush>("TextFillColorPrimaryBrush"),
                VerticalAlignment = VerticalAlignment.Center,
                Margin = new Thickness(0, 0, 10, 0),
                FontWeight = FontWeights.SemiBold
            };

            ToggleSwitch DiscordRefresh = new()
            {
                IsChecked = SMMH.DiscordRefresh
            };

            DiscordRefresh.Checked += (s, e) => DiscordRefreshChecked(true);
            DiscordRefresh.Unchecked += (s, e) => DiscordRefreshChecked(false);

            StackPanel DiscordDelayContent = new()
            {
                Orientation = Orientation.Horizontal,
                Margin = new Thickness(0, 10, 0, 0)
            };

            TextBlock DiscordDelayText = new()
            {
                Text = SRER.GetValue("Portal", "OtherSettingPage", "DiscordHook", "DiscordDelay"),
                Foreground = SRER.GetResource<Brush>("TextFillColorPrimaryBrush"),
                VerticalAlignment = VerticalAlignment.Center,
                Margin = new Thickness(0, 0, 10, 0),
                FontWeight = FontWeights.SemiBold
            };

            NumberBox DiscordDelay = new()
            {
                Icon = new SymbolIcon(SymbolRegular.TimePicker24),
                IconPlacement = ElementPlacement.Left,
                ClearButtonEnabled = false,
                Value = SMMH.DiscordDelay,
                MaxDecimalPlaces = 0,
                Maximum = 3600,
                MaxLength = 4,
                Minimum = 60
            };

            DiscordDelay.ValueChanged += (s, e) => DiscordDelayChanged(DiscordDelay.Value);

            DiscordRefreshContent.Children.Add(DiscordRefreshText);
            DiscordRefreshContent.Children.Add(DiscordRefresh);

            DiscordDelayContent.Children.Add(DiscordDelayText);
            DiscordDelayContent.Children.Add(DiscordDelay);

            DiscordContent.Children.Add(DiscordRefreshContent);
            DiscordContent.Children.Add(DiscordDelayContent);

            DiscordHook.FooterCard = DiscordContent;

            Contents.Add(DiscordHook);

            TextBlock PriorityArea = new()
            {
                Foreground = SRER.GetResource<Brush>("TextFillColorPrimaryBrush"),
                Text = SRER.GetValue("Portal", "Area", "Priority"),
                Margin = new Thickness(0, 10, 0, 0),
                FontWeight = FontWeights.Bold
            };

            Contents.Add(PriorityArea);

            SPVCEC Agent = new()
            {
                Margin = new Thickness(0, 10, 0, 0),
                Expandable = false
            };

            Agent.LeftIcon.Symbol = SymbolRegular.VideoPersonSparkle24;
            Agent.Title.Text = SRER.GetValue("Portal", "OtherSettingPage", "Agent");
            Agent.Description.Text = SRER.GetValue("Portal", "OtherSettingPage", "Agent", "Description");

            TextBox UserAgent = new()
            {
                Icon = new SymbolIcon(SymbolRegular.PersonHeart24),
                IconPlacement = ElementPlacement.Left,
                ClearButtonEnabled = false,
                Text = SMMG.UserAgent,
                IsReadOnly = true,
                MaxLength = 100,
                MinWidth = 125,
                MaxWidth = 250
            };

            UserAgent.TextChanged += (s, e) => UserAgentChanged(UserAgent);

            Agent.HeaderFrame = UserAgent;

            Contents.Add(Agent);

            SPVCEC Key = new()
            {
                Margin = new Thickness(0, 10, 0, 0)
            };

            Key.LeftIcon.Symbol = SymbolRegular.ShieldKeyhole24;
            Key.Title.Text = SRER.GetValue("Portal", "OtherSettingPage", "Key");
            Key.Description.Text = SRER.GetValue("Portal", "OtherSettingPage", "Key", "Description");

            StackPanel KeyContent = new();

            HyperlinkButton HintKey = new()
            {
                Content = SRER.GetValue("Portal", "OtherSettingPage", "Key", "HintKey"),
                Foreground = SRER.GetResource<Brush>("AccentTextFillColorPrimaryBrush"),
                NavigateUri = SMMRU.YouTubePersonalAccessToken,
                Appearance = ControlAppearance.Transparent,
                BorderBrush = Brushes.Transparent,
                Cursor = Cursors.Hand
            };

            TextBox PersonalKey = new()
            {
                PlaceholderText = SRER.GetValue("Portal", "OtherSettingPage", "Key", "PersonalKey"),
                Icon = new SymbolIcon(SymbolRegular.PersonKey20),
                HorizontalAlignment = HorizontalAlignment.Left,
                IconPlacement = ElementPlacement.Left,
                Margin = new Thickness(0, 10, 0, 0),
                Text = SMMO.Key,
                MaxLength = 93
            };

            PersonalKey.TextChanged += (s, e) => PersonalKeyChanged(PersonalKey);

            KeyContent.Children.Add(HintKey);
            KeyContent.Children.Add(PersonalKey);

            Key.FooterCard = KeyContent;

            Contents.Add(Key);

            TextBlock UpdateArea = new()
            {
                Foreground = SRER.GetResource<Brush>("TextFillColorPrimaryBrush"),
                Text = SRER.GetValue("Portal", "Area", "Update"),
                Margin = new Thickness(0, 10, 0, 0),
                FontWeight = FontWeights.Bold
            };

            Contents.Add(UpdateArea);

            SPVCEC Server = new()
            {
                Margin = new Thickness(0, 10, 0, 0)
            };

            Server.LeftIcon.Symbol = SymbolRegular.ServerSurfaceMultiple16; //SymbolRegular.CloudFlow24
            Server.Title.Text = SRER.GetValue("Portal", "OtherSettingPage", "Server");
            Server.Description.Text = SRER.GetValue("Portal", "OtherSettingPage", "Server", "Description");

            ComboBox ServerType = new();

            ServerType.SelectionChanged += (s, e) => ServerTypeSelected(ServerType.SelectedIndex);

            foreach (SSDEUST Type in Enum.GetValues(typeof(SSDEUST)))
            {
                ServerType.Items.Add(new ComboBoxItem()
                {
                    Content = SRER.GetValue("Portal", "Enum", "UpdateServerType", $"{Type}")
                });
            }

            ServerType.SelectedIndex = (int)SSDMMU.UpdateServerType;

            Server.HeaderFrame = ServerType;

            TextBlock ServerHint = new()
            {
                Text = string.Format(SRER.GetValue("Portal", "OtherSettingPage", "Server", "ServerHint"), SRER.GetValue("Portal", "Enum", "UpdateServerType", $"{SSDEUST.GitHub}"), SRER.GetValue("Portal", "Enum", "UpdateServerType", $"{SSDEUST.Soferity}"), SRER.GetValue("Portal", "OtherSettingPage", "Key")),
                Foreground = SRER.GetResource<Brush>("TextFillColorSecondaryBrush"),
                HorizontalAlignment = HorizontalAlignment.Left,
                TextWrapping = TextWrapping.WrapWithOverflow,
                TextAlignment = TextAlignment.Left,
                Margin = new Thickness(0, 0, 0, 0),
                FontWeight = FontWeights.SemiBold
            };

            Server.FooterCard = ServerHint;

            Contents.Add(Server);

            SPVCEC Module = new()
            {
                Margin = new Thickness(0, 10, 0, 0),
                Expandable = false
            };

            Module.LeftIcon.Symbol = SymbolRegular.BoxToolbox24;
            Module.Title.Text = SRER.GetValue("Portal", "OtherSettingPage", "Module");
            Module.Description.Text = SRER.GetValue("Portal", "OtherSettingPage", "Module", "Description");

            ComboBox ModuleType = new();

            ModuleType.SelectionChanged += (s, e) => ModuleTypeSelected(ModuleType.SelectedIndex);

            foreach (SSDEUMT Type in Enum.GetValues(typeof(SSDEUMT)))
            {
                ModuleType.Items.Add(SRER.GetValue("Portal", "Enum", "UpdateModuleType", $"{Type}"));
            }

            ModuleType.SelectedIndex = (int)SSDMMU.UpdateModuleType;

            Module.HeaderFrame = ModuleType;

            Contents.Add(Module);

            SPVCEC Channel = new()
            {
                Margin = new Thickness(0, 10, 0, 0),
                Expandable = false
            };

            Channel.LeftIcon.Symbol = SymbolRegular.Production24;
            Channel.Title.Text = SRER.GetValue("Portal", "OtherSettingPage", "Channel");
            Channel.Description.Text = SRER.GetValue("Portal", "OtherSettingPage", "Channel", "Description");

            ComboBox ChannelType = new();

            ChannelType.SelectionChanged += (s, e) => ChannelTypeSelected(ChannelType.SelectedIndex);

            foreach (SSCEUCT Type in Enum.GetValues(typeof(SSCEUCT)))
            {
                ChannelType.Items.Add(SRER.GetValue("Portal", "Enum", "UpdateChannelType", $"{Type}"));
            }

            ChannelType.SelectedIndex = (int)SSCMMU.UpdateChannelType;

            Channel.HeaderFrame = ChannelType;

            Contents.Add(Channel);

            SPVCEC Update = new()
            {
                Margin = new Thickness(0, 10, 0, 0),
                IsExpand = true
            };

            Update.LeftIcon.Symbol = SymbolRegular.BoxMultiple24;
            Update.Title.Text = SRER.GetValue("Portal", "OtherSettingPage", "Update");
            Update.Description.Text = SRER.GetValue("Portal", "OtherSettingPage", "Update", "Description");

            ComboBox UpdateType = new();

            UpdateType.SelectionChanged += (s, e) => UpdateTypeSelected(UpdateType.SelectedIndex);

            foreach (SSCEUET Type in Enum.GetValues(typeof(SSCEUET)))
            {
                UpdateType.Items.Add(SRER.GetValue("Portal", "Enum", "UpdateExtensionType", $"{Type}"));
            }

            UpdateType.SelectedIndex = (int)SSCMMU.UpdateExtensionType;

            Update.HeaderFrame = UpdateType;

            StackPanel UpdateContent = new()
            {
                Orientation = Orientation.Horizontal
            };

            TextBlock UpdateLimitText = new()
            {
                Text = SRER.GetValue("Portal", "OtherSettingPage", "Update", "Limit"),
                Foreground = SRER.GetResource<Brush>("TextFillColorPrimaryBrush"),
                VerticalAlignment = VerticalAlignment.Center,
                Margin = new Thickness(0, 0, 10, 0),
                FontWeight = FontWeights.SemiBold
            };

            NumberBox UpdateLimit = new()
            {
                Icon = new SymbolIcon(SymbolRegular.ArrowBetweenDown24),
                IconPlacement = ElementPlacement.Left,
                Margin = new Thickness(0, 0, 10, 0),
                Value = SMMU.UpdateLimitValue,
                ClearButtonEnabled = false,
                MaxDecimalPlaces = 0,
                Maximum = 99999999,
                MaxLength = 8,
                Minimum = 0
            };

            UpdateLimit.ValueChanged += (s, e) => UpdateLimitChanged(UpdateLimit.Value);

            ComboBox UpdateLimitType = new()
            {
                MaxDropDownHeight = 200
            };

            DynamicScrollViewer.SetVerticalScrollBarVisibility(UpdateLimitType, ScrollBarVisibility.Auto);

            UpdateLimitType.SelectionChanged += (s, e) => UpdateLimitTypeSelected(UpdateLimitType.SelectedIndex);

            foreach (SEST Type in Enum.GetValues(typeof(SEST)))
            {
                if (Type >= SEST.Megabyte)
                {
                    UpdateLimitType.Items.Add(new ComboBoxItem()
                    {
                        IsEnabled = true,
                        Content = Type
                    });
                }
                else
                {
                    UpdateLimitType.Items.Add(new ComboBoxItem()
                    {
                        IsEnabled = false,
                        Content = Type
                    });
                }
            }

            UpdateLimitType.SelectedIndex = (int)SMMB.DownloadType;

            UpdateContent.Children.Add(UpdateLimitText);
            UpdateContent.Children.Add(UpdateLimit);
            UpdateContent.Children.Add(UpdateLimitType);

            Update.FooterCard = UpdateContent;

            Contents.Add(Update);

            SPVCEC Auto = new()
            {
                Margin = new Thickness(0, 10, 0, 0),
                Expandable = false
            };

            Auto.LeftIcon.Symbol = SymbolRegular.CubeArrowCurveDown20; //Component2DoubleTapSwipeDown24 - ArrowCircleDownDouble24
            Auto.Title.Text = SRER.GetValue("Portal", "OtherSettingPage", "Auto");
            Auto.Description.Text = SRER.GetValue("Portal", "OtherSettingPage", "Auto", "Description");

            ToggleSwitch AutoState = new()
            {
                IsChecked = SMMU.AutoUpdate
            };

            AutoState.Checked += (s, e) => AutoStateChecked(true);
            AutoState.Unchecked += (s, e) => AutoStateChecked(false);

            Auto.HeaderFrame = AutoState;

            Contents.Add(Auto);

            TextBlock DeveloperArea = new()
            {
                Foreground = SRER.GetResource<Brush>("TextFillColorPrimaryBrush"),
                Text = SRER.GetValue("Portal", "Area", "Developer"),
                Margin = new Thickness(0, 10, 0, 0),
                FontWeight = FontWeights.Bold
            };

            Contents.Add(DeveloperArea);

            SPVCEC Developer = new()
            {
                Margin = new Thickness(0, 10, 0, 0),
                Expandable = true,
                IsExpand = true
            };

            Developer.LeftIcon.Symbol = SymbolRegular.WindowDevTools24;
            Developer.Title.Text = SRER.GetValue("Portal", "OtherSettingPage", "Developer");
            Developer.Description.Text = SRER.GetValue("Portal", "OtherSettingPage", "Developer", "Description");

            StackPanel DeveloperContent = new();

            StackPanel DeveloperModeContent = new()
            {
                Orientation = Orientation.Horizontal
            };

            TextBlock DeveloperModeText = new()
            {
                Text = SRER.GetValue("Portal", "OtherSettingPage", "Developer", "DeveloperMode"),
                Foreground = SRER.GetResource<Brush>("TextFillColorPrimaryBrush"),
                VerticalAlignment = VerticalAlignment.Center,
                Margin = new Thickness(0, 0, 10, 0),
                FontWeight = FontWeights.SemiBold
            };

            ToggleSwitch DeveloperMode = new()
            {
                IsChecked = SMME.DeveloperMode
            };

            DeveloperMode.Checked += (s, e) => DeveloperModeChecked(true);
            DeveloperMode.Unchecked += (s, e) => DeveloperModeChecked(false);

            StackPanel DeveloperPortContent = new()
            {
                Orientation = Orientation.Horizontal,
                Margin = new Thickness(0, 10, 0, 0)
            };

            TextBlock DeveloperPortText = new()
            {
                Text = SRER.GetValue("Portal", "OtherSettingPage", "Developer", "DeveloperPort"),
                Foreground = SRER.GetResource<Brush>("TextFillColorPrimaryBrush"),
                VerticalAlignment = VerticalAlignment.Center,
                Margin = new Thickness(0, 0, 10, 0),
                FontWeight = FontWeights.SemiBold
            };

            NumberBox DeveloperPort = new()
            {
                Icon = new SymbolIcon(SymbolRegular.SerialPort24),
                IconPlacement = ElementPlacement.Left,
                Value = SMME.DeveloperPort,
                ClearButtonEnabled = false,
                MaxDecimalPlaces = 0,
                Maximum = 65535,
                MaxLength = 5,
                Minimum = 0
            };

            TextBlock DeveloperHint = new()
            {
                Text = string.Format(SRER.GetValue("Portal", "OtherSettingPage", "Developer", "DeveloperHint"), SMME.DeveloperPort),
                Foreground = SRER.GetResource<Brush>("TextFillColorSecondaryBrush"),
                HorizontalAlignment = HorizontalAlignment.Left,
                TextWrapping = TextWrapping.WrapWithOverflow,
                Margin = new Thickness(0, 10, 0, 0),
                TextAlignment = TextAlignment.Left,
                FontWeight = FontWeights.SemiBold
            };

            DeveloperPort.ValueChanged += (s, e) => DeveloperPortChanged(DeveloperHint, DeveloperPort.Value);

            DeveloperModeContent.Children.Add(DeveloperModeText);
            DeveloperModeContent.Children.Add(DeveloperMode);

            DeveloperPortContent.Children.Add(DeveloperPortText);
            DeveloperPortContent.Children.Add(DeveloperPort);

            DeveloperContent.Children.Add(DeveloperModeContent);
            DeveloperContent.Children.Add(DeveloperPortContent);
            DeveloperContent.Children.Add(DeveloperHint);

            Developer.FooterCard = DeveloperContent;

            Contents.Add(Developer);

            _isInitialized = true;
        }

        private void AutoStateChecked(bool State)
        {
            SMMI.UpdateSettingManager.SetSetting(SMMCU.AutoUpdate, State);
        }

        private void ModuleTypeSelected(int Index)
        {
            if (Index != (int)SSDMMU.UpdateModuleType)
            {
                SSDEUMT Type = (SSDEUMT)Index;

                SMMI.UpdateSettingManager.SetSetting(SMMCU.UpdateModuleType, Type);
            }
        }

        private void ServerTypeSelected(int Index)
        {
            if (Index != (int)SSDMMU.UpdateServerType)
            {
                SSDEUST Type = (SSDEUST)Index;

                SMMI.UpdateSettingManager.SetSetting(SMMCU.UpdateServerType, Type);
            }
        }

        private void UpdateTypeSelected(int Index)
        {
            if (Index != (int)SSCMMU.UpdateExtensionType)
            {
                SSCEUET Type = (SSCEUET)Index;

                SMMI.UpdateSettingManager.SetSetting(SMMCU.UpdateExtensionType, Type);
            }
        }

        private void ReportStateChecked(bool State)
        {
            SMMI.GeneralSettingManager.SetSetting(SMMCG.Report, State);

            if (State || SMMG.Statistics)
            {
                SSSHP.Run(SSSPMI.Commandog, $"{SMMRG.StartCommand}{SSDECT.Reportdog}{SMMRG.ValueSeparator}{SSSPMI.Reportdog}");
            }
            else if (!SMMG.Statistics)
            {
                if (SSSHP.Work(SMMRA.Reportdog))
                {
                    SSSHP.Kill(SMMRA.Reportdog);
                }
            }
        }

        private void ChannelTypeSelected(int Index)
        {
            if (Index != (int)SSCMMU.UpdateChannelType)
            {
                SSCEUCT Type = (SSCEUCT)Index;

                SMMI.UpdateSettingManager.SetSetting(SMMCU.UpdateChannelType, Type);
            }
        }

        private void DiscordStateChecked(bool State)
        {
            SMMI.HookSettingManager.SetSetting(SMMCH.DiscordState, State);
        }

        private void DeveloperModeChecked(bool State)
        {
            SMMI.EngineSettingManager.SetSetting(SMMCE.DeveloperMode, State);
        }

        private void DiscordRefreshChecked(bool State)
        {
            SMMI.HookSettingManager.SetSetting(SMMCH.DiscordRefresh, State);
        }

        private void UserAgentChanged(TextBox TextBox)
        {
            if (string.IsNullOrEmpty(TextBox.Text))
            {
                TextBox.Text = SMMRG.UserAgent;
            }

            SMMI.GeneralSettingManager.SetSetting(SMMCG.UserAgent, TextBox.Text);
        }

        private void UpdateLimitChanged(double? Value)
        {
            int NewValue = Convert.ToInt32(Value);

            if (NewValue != SMMU.UpdateLimitValue)
            {
                SMMI.UpdateSettingManager.SetSetting(SMMCU.UpdateLimitValue, NewValue);
            }
        }

        private void StatisticsStateChecked(bool State)
        {
            SMMI.GeneralSettingManager.SetSetting(SMMCG.Statistics, State);

            if (State || SMMG.Report)
            {
                SSSHP.Run(SSSPMI.Commandog, $"{SMMRG.StartCommand}{SSDECT.Reportdog}{SMMRG.ValueSeparator}{SSSPMI.Reportdog}");
            }
            else if (!SMMG.Report)
            {
                if (SSSHP.Work(SMMRA.Reportdog))
                {
                    SSSHP.Kill(SMMRA.Reportdog);
                }
            }
        }

        private void UpdateLimitTypeSelected(int Index)
        {
            if (Index != (int)SMMU.UpdateLimitType)
            {
                SEST Type = (SEST)Index;

                SMMI.UpdateSettingManager.SetSetting(SMMCU.UpdateLimitType, Type);
            }
        }

        private void DiscordDelayChanged(double? Value)
        {
            int NewValue = Convert.ToInt32(Value);

            if (NewValue != SMMH.DiscordDelay)
            {
                SMMI.HookSettingManager.SetSetting(SMMCH.DiscordDelay, NewValue);
            }
        }

        private void PersonalKeyChanged(TextBox TextBox)
        {
            if (TextBox.Text.Length == 93)
            {
                SSSTMI.State = true;
                SMMI.ObjectionableSettingManager.SetSetting(SMMCO.Key, TextBox.Text);
            }
            else
            {
                SMMI.ObjectionableSettingManager.SetSetting(SMMCO.Key, string.Empty);
                TextBox.PlaceholderText = SRER.GetValue("Portal", "OtherSettingPage", "Key", "PersonalKey", "Valid");
            }
        }

        private void DeveloperPortChanged(TextBlock Developer, double? Value)
        {
            int NewValue = Convert.ToInt32(Value);

            if (NewValue != SMME.DeveloperPort)
            {
                SMMI.EngineSettingManager.SetSetting(SMMCE.DeveloperPort, NewValue);

                Developer.Text = string.Format(SRER.GetValue("Portal", "OtherSettingPage", "Developer", "DeveloperHint"), NewValue);
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