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
using SPVCEC = Sucrose.Portal.Views.Controls.ExpanderCard;
using SSRER = Sucrose.Shared.Resources.Extension.Resources;
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
                Margin = new Thickness(0, 0, 0, 0),
                FontWeight = FontWeights.Bold,
                Text = "Kanca"
            };

            Contents.Add(HookArea);

            SPVCEC DiscordHook = new()
            {
                Margin = new Thickness(0, 10, 0, 0),
                IsExpand = true
            };

            DiscordHook.Title.Text = "Discord";
            DiscordHook.LeftIcon.Symbol = SymbolRegular.SquareHintApps24;
            DiscordHook.Description.Text = "Sucrose, Discord ile arasındaki bağlantıyı sağlar.";

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
                Foreground = SSRER.GetResource<Brush>("TextFillColorPrimaryBrush"),
                VerticalAlignment = VerticalAlignment.Center,
                Margin = new Thickness(0, 0, 10, 0),
                FontWeight = FontWeights.SemiBold,
                Text = "Bağlantı Yenileme:",
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
                Foreground = SSRER.GetResource<Brush>("TextFillColorPrimaryBrush"),
                VerticalAlignment = VerticalAlignment.Center,
                Margin = new Thickness(0, 0, 10, 0),
                Text = "Yenileme Süresi (Saniye):",
                FontWeight = FontWeights.SemiBold
            };

            NumberBox DiscordDelay = new()
            {
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
                Margin = new Thickness(0, 10, 0, 0),
                FontWeight = FontWeights.Bold,
                Text = "Öncelik"
            };

            Contents.Add(PriorityArea);

            SPVCEC Agent = new()
            {
                Margin = new Thickness(0, 10, 0, 0),
                Expandable = false
            };

            Agent.Title.Text = "İnternet Temsilcisi";
            Agent.LeftIcon.Symbol = SymbolRegular.VideoPersonSparkle24;
            Agent.Description.Text = "İnternet bağlantısı gerektiren durumlardaki temsilci kimliğiniz.";

            TextBox UserAgent = new()
            {
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

            Key.Title.Text = "Kişisel Erişim Belirteci";
            Key.LeftIcon.Symbol = SymbolRegular.ShieldKeyhole24;
            Key.Description.Text = "Mağazayı ve uygulama güncellemeyi sorunsuz bir şekilde kullanmak için gerekli.";

            StackPanel KeyContent = new();

            Hyperlink HintKey = new()
            {
                Content = "Kişisel erişim belirtecini nasıl elde edeceğinizi bilmiyorsanız buraya tıklayın.",
                Foreground = SSRER.GetResource<Brush>("AccentTextFillColorPrimaryBrush"),
                Appearance = ControlAppearance.Transparent,
                BorderBrush = Brushes.Transparent,
                NavigateUri = SMR.KeyYouTube,
                Cursor = Cursors.Hand
            };

            TextBox PersonalKey = new()
            {
                PlaceholderText = "Lütfen bir kişisel erişim belirteci girin",
                HorizontalAlignment = HorizontalAlignment.Left,
                Margin = new Thickness(0, 10, 0, 0),
                Text = SMMM.Key,
                MaxLength = 93
            };

            PersonalKey.TextChanged += (s, e) => PersonalKeyChanged(PersonalKey);

            KeyContent.Children.Add(HintKey);
            KeyContent.Children.Add(PersonalKey);

            Key.FooterCard = KeyContent;

            Contents.Add(Key);

            TextBlock DeveloperArea = new()
            {
                Foreground = SSRER.GetResource<Brush>("TextFillColorPrimaryBrush"),
                Margin = new Thickness(0, 10, 0, 0),
                FontWeight = FontWeights.Bold,
                Text = "Geliştirici"
            };

            Contents.Add(DeveloperArea);

            SPVCEC Developer = new()
            {
                Margin = new Thickness(0, 10, 0, 0),
                Expandable = false
            };

            Developer.Title.Text = "Geliştirici Araçları";
            Developer.LeftIcon.Symbol = SymbolRegular.WindowDevTools24;
            Developer.Description.Text = "WebView ve CefSharp motoru için geliştirici araçlarının gösterilip gösterilmeyeceği.";

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
            }
            else
            {
                SMMI.PrivateSettingManager.SetSetting(SMC.Key, SMR.Key);
                TextBox.PlaceholderText = "Lütfen geçerli bir kişisel erişim belirteci girin";
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