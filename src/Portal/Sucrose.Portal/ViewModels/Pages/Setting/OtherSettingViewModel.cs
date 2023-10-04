using CommunityToolkit.Mvvm.ComponentModel;
using System.Windows;
using System.Windows.Controls;
using System.Windows.Input;
using System.Windows.Media;
using Wpf.Ui.Controls;
using SMC = Sucrose.Memory.Constant;
using SMMI = Sucrose.Manager.Manage.Internal;
using SMMM = Sucrose.Manager.Manage.Manager;
using SMR = Sucrose.Memory.Readonly;
using SPMM = Sucrose.Portal.Manage.Manager;
using SPVCEC = Sucrose.Portal.Views.Controls.ExpanderCard;
using SSCEUT = Sucrose.Shared.Core.Enum.UpdateType;
using SSRER = Sucrose.Shared.Resources.Extension.Resources;
using SSSMI = Sucrose.Shared.Store.Manage.Internal;
using TextBlock = System.Windows.Controls.TextBlock;
using TextBox = Wpf.Ui.Controls.TextBox;

namespace Sucrose.Portal.ViewModels.Pages
{
    public partial class OtherSettingViewModel : ObservableObject, INavigationAware, IDisposable
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
            TextBlock HookArea = new()
            {
                Foreground = SSRER.GetResource<Brush>("TextFillColorPrimaryBrush"),
                Text = SSRER.GetValue("Portal", "Area", "Hook"),
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
            DiscordHook.Title.Text = SSRER.GetValue("Portal", "OtherSettingPage", "DiscordHook");
            DiscordHook.Description.Text = SSRER.GetValue("Portal", "OtherSettingPage", "DiscordHook", "Description");

            ToggleSwitch DiscordState = new()
            {
                IsChecked = SMMM.DiscordState
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
                Text = SSRER.GetValue("Portal", "OtherSettingPage", "DiscordHook", "DiscordRefresh"),
                Foreground = SSRER.GetResource<Brush>("TextFillColorPrimaryBrush"),
                VerticalAlignment = VerticalAlignment.Center,
                Margin = new Thickness(0, 0, 10, 0),
                FontWeight = FontWeights.SemiBold
            };

            ToggleSwitch DiscordRefresh = new()
            {
                IsChecked = SMMM.DiscordRefresh
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
                Text = SSRER.GetValue("Portal", "OtherSettingPage", "DiscordHook", "DiscordDelay"),
                Foreground = SSRER.GetResource<Brush>("TextFillColorPrimaryBrush"),
                VerticalAlignment = VerticalAlignment.Center,
                Margin = new Thickness(0, 0, 10, 0),
                FontWeight = FontWeights.SemiBold
            };

            NumberBox DiscordDelay = new()
            {
                Icon = new SymbolIcon(SymbolRegular.TimePicker24),
                IconPlacement = ElementPlacement.Left,
                ClearButtonEnabled = false,
                Value = SMMM.DiscordDelay,
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
                Foreground = SSRER.GetResource<Brush>("TextFillColorPrimaryBrush"),
                Text = SSRER.GetValue("Portal", "Area", "Priority"),
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
            Agent.Title.Text = SSRER.GetValue("Portal", "OtherSettingPage", "Agent");
            Agent.Description.Text = SSRER.GetValue("Portal", "OtherSettingPage", "Agent", "Description");

            TextBox UserAgent = new()
            {
                Icon = new SymbolIcon(SymbolRegular.PersonHeart24),
                IconPlacement = ElementPlacement.Left,
                ClearButtonEnabled = false,
                Text = SMMM.UserAgent,
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
            Key.Title.Text = SSRER.GetValue("Portal", "OtherSettingPage", "Key");
            Key.Description.Text = SSRER.GetValue("Portal", "OtherSettingPage", "Key", "Description");

            StackPanel KeyContent = new();

            Hyperlink HintKey = new()
            {
                Content = SSRER.GetValue("Portal", "OtherSettingPage", "Key", "HintKey"),
                Foreground = SSRER.GetResource<Brush>("AccentTextFillColorPrimaryBrush"),
                Appearance = ControlAppearance.Transparent,
                BorderBrush = Brushes.Transparent,
                NavigateUri = SMR.KeyYouTube,
                Cursor = Cursors.Hand
            };

            TextBox PersonalKey = new()
            {
                PlaceholderText = SSRER.GetValue("Portal", "OtherSettingPage", "Key", "PersonalKey"),
                Icon = new SymbolIcon(SymbolRegular.PersonKey20),
                HorizontalAlignment = HorizontalAlignment.Left,
                IconPlacement = ElementPlacement.Left,
                Margin = new Thickness(0, 10, 0, 0),
                Text = SMMM.Key,
                MaxLength = 93
            };

            PersonalKey.TextChanged += (s, e) => PersonalKeyChanged(PersonalKey);

            KeyContent.Children.Add(HintKey);
            KeyContent.Children.Add(PersonalKey);

            Key.FooterCard = KeyContent;

            Contents.Add(Key);

            TextBlock UpdateArea = new()
            {
                Foreground = SSRER.GetResource<Brush>("TextFillColorPrimaryBrush"),
                Text = SSRER.GetValue("Portal", "Area", "Update"),
                Margin = new Thickness(0, 10, 0, 0),
                FontWeight = FontWeights.Bold
            };

            Contents.Add(UpdateArea);

            SPVCEC Update = new()
            {
                Margin = new Thickness(0, 10, 0, 0),
                Expandable = false
            };

            Update.LeftIcon.Symbol = SymbolRegular.ArrowSwap24;
            Update.Title.Text = SSRER.GetValue("Portal", "OtherSettingPage", "Update");
            Update.Description.Text = SSRER.GetValue("Portal", "OtherSettingPage", "Update", "Description");

            ComboBox UpdateType = new();

            UpdateType.SelectionChanged += (s, e) => UpdateTypeSelected(UpdateType.SelectedIndex);

            foreach (SSCEUT Type in Enum.GetValues(typeof(SSCEUT)))
            {
                UpdateType.Items.Add(SSRER.GetValue("Portal", "Enum", "UpdateType", $"{Type}"));
            }

            UpdateType.SelectedIndex = (int)SPMM.UpdateType;

            Update.HeaderFrame = UpdateType;

            Contents.Add(Update);

            TextBlock DeveloperArea = new()
            {
                Foreground = SSRER.GetResource<Brush>("TextFillColorPrimaryBrush"),
                Text = SSRER.GetValue("Portal", "Area", "Developer"),
                Margin = new Thickness(0, 10, 0, 0),
                FontWeight = FontWeights.Bold
            };

            Contents.Add(DeveloperArea);

            SPVCEC Developer = new()
            {
                Margin = new Thickness(0, 10, 0, 0),
                Expandable = false
            };

            Developer.LeftIcon.Symbol = SymbolRegular.WindowDevTools24;
            Developer.Title.Text = SSRER.GetValue("Portal", "OtherSettingPage", "Developer");
            Developer.Description.Text = SSRER.GetValue("Portal", "OtherSettingPage", "Developer", "Description");

            ToggleSwitch DeveloperState = new()
            {
                IsChecked = SMMM.DeveloperMode
            };

            DeveloperState.Checked += (s, e) => DeveloperStateChecked(true);
            DeveloperState.Unchecked += (s, e) => DeveloperStateChecked(false);

            Developer.HeaderFrame = DeveloperState;

            Contents.Add(Developer);

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

        private void UpdateTypeSelected(int Index)
        {
            if (Index != (int)SPMM.UpdateType)
            {
                SSCEUT Type = (SSCEUT)Index;

                SMMI.UpdateSettingManager.SetSetting(SMC.UpdateType, Type);
            }
        }

        private void DiscordStateChecked(bool State)
        {
            SMMI.HookSettingManager.SetSetting(SMC.DiscordState, State);
        }

        private void DiscordRefreshChecked(bool State)
        {
            SMMI.HookSettingManager.SetSetting(SMC.DiscordRefresh, State);
        }

        private void UserAgentChanged(TextBox TextBox)
        {
            if (string.IsNullOrEmpty(TextBox.Text))
            {
                TextBox.Text = SMR.UserAgent;
            }

            SMMI.GeneralSettingManager.SetSetting(SMC.UserAgent, TextBox.Text);
        }

        private void DeveloperStateChecked(bool State)
        {
            SMMI.EngineSettingManager.SetSetting(SMC.DeveloperMode, State);
        }

        private void DiscordDelayChanged(double? Value)
        {
            int NewValue = Convert.ToInt32(Value);

            if (NewValue != SMMM.DiscordDelay)
            {
                SMMI.HookSettingManager.SetSetting(SMC.DiscordDelay, NewValue);
            }
        }

        private void PersonalKeyChanged(TextBox TextBox)
        {
            if (TextBox.Text.Length == 93)
            {
                SMMI.PrivateSettingManager.SetSetting(SMC.Key, TextBox.Text);
                SSSMI.State = true;
            }
            else
            {
                SMMI.PrivateSettingManager.SetSetting(SMC.Key, SMR.Key);
                TextBox.PlaceholderText = SSRER.GetValue("Portal", "OtherSettingPage", "Key", "PersonalKey", "Valid");
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