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
    public partial class PersonalSettingViewModel : ObservableObject, INavigationAware, IDisposable
    {
        [ObservableProperty]
        private List<UIElement> _Contents = new();

        private bool _isInitialized;

        public PersonalSettingViewModel()
        {
            if (!_isInitialized)
            {
                InitializeViewModel();
            }
        }

        private void InitializeViewModel()
        {
            TextBlock AppearanceBehavior = new()
            {
                Foreground = SSRER.GetResource<Brush>("TextFillColorPrimaryBrush"),
                Margin = new Thickness(0, 0, 0, 0),
                FontWeight = FontWeights.Bold,
                Text = "Görünüş & Davranış"
            };

            Contents.Add(AppearanceBehavior);

            SPVCEC ApplicationLanguage = new()
            {
                Margin = new Thickness(0, 10, 0, 0),
                Expandable = false
            };

            ApplicationLanguage.Title.Text = "Uygulama Dili";
            ApplicationLanguage.LeftIcon.Symbol = SymbolRegular.LocalLanguage24;
            ApplicationLanguage.Description.Text = "Uygulamayı görüntülemek istediğiniz dili seçin.";

            TextBlock Library = new()
            {
                Foreground = SSRER.GetResource<Brush>("TextFillColorPrimaryBrush"),
                Margin = new Thickness(0, 10, 0, 0),
                FontWeight = FontWeights.Bold,
                Text = "Kütüphane"
            };

            Contents.Add(Library);

            SPVCEC Confirm = new()
            {
                Margin = new Thickness(0, 10, 0, 0),
                Expandable = false
            };

            Confirm.Title.Text = "Silme Onayı";
            Confirm.LeftIcon.Symbol = SymbolRegular.DeleteDismiss24;
            Confirm.Description.Text = "Kütüphanedeki temaları silerken onay isteyip istemeyeceği.";

            ToggleSwitch ConfirmState = new()
            {
                IsChecked = SMMM.LibraryConfirm
            };

            ConfirmState.Checked += (s, e) => ConfirmStateChecked(true);
            ConfirmState.Unchecked += (s, e) => ConfirmStateChecked(false);

            Confirm.HeaderFrame = ConfirmState;

            Contents.Add(Confirm);

            TextBlock Store = new()
            {
                Foreground = SSRER.GetResource<Brush>("TextFillColorPrimaryBrush"),
                Margin = new Thickness(0, 10, 0, 0),
                FontWeight = FontWeights.Bold,
                Text = "Mağaza"
            };

            Contents.Add(Store);

            SPVCEC Adult = new()
            {
                Margin = new Thickness(0, 10, 0, 0),
                Expandable = false
            };

            Adult.Title.Text = "Zararlı İçerikler";
            Adult.LeftIcon.Symbol = SymbolRegular.ContentSettings24;
            Adult.Description.Text = "Mağazada zararlı (NSFW) olarak işaretlenmiş içeriklerin gösterilmesi.";

            ToggleSwitch AdultState = new()
            {
                IsChecked = SMMM.Adult
            };

            AdultState.Checked += (s, e) => AdultStateChecked(true);
            AdultState.Unchecked += (s, e) => AdultStateChecked(false);

            Adult.HeaderFrame = AdultState;

            Contents.Add(Adult);

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

            Key.Title.Text = "GitHub API Anahtarı";
            Key.LeftIcon.Symbol = SymbolRegular.ShieldKeyhole24;
            Key.Description.Text = "Mağazayı ve uygulama güncellemeyi sorunsuz bir şekilde kullanmak için gerekli.";

            StackPanel KeyContent = new();

            Hyperlink HintKey = new()
            {
                Content = "API anahtarını nasıl elde edeceğinizi bilmiyorsanız buraya tıklayın.",
                Foreground = SSRER.GetResource<Brush>("AccentTextFillColorPrimaryBrush"),
                Appearance = ControlAppearance.Transparent,
                BorderBrush = Brushes.Transparent,
                NavigateUri = SMR.KeyWebsite,
                Cursor = Cursors.Hand
            };

            TextBox PersonalKey = new()
            {
                PlaceholderText = "Lütfen bir API Anahtarı girin",
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

        private void UserAgentChanged(TextBox TextBox)
        {
            if (string.IsNullOrEmpty(TextBox.Text))
            {
                TextBox.Text = SMR.UserAgent;
            }

            SMMI.GeneralSettingManager.SetSetting(SMC.UserAgent, TextBox.Text);
        }

        private void AdultStateChecked(bool State)
        {
            SMMI.PortalSettingManager.SetSetting(SMC.Adult, State);
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
                TextBox.PlaceholderText = "Lütfen geçerli bir API Anahtarı girin";
            }
        }

        private void ConfirmStateChecked(bool State)
        {
            SMMI.LibrarySettingManager.SetSetting(SMC.LibraryConfirm, State);
        }

        public void Dispose()
        {
            Contents.Clear();

            GC.Collect();
            GC.SuppressFinalize(this);
        }
    }
}