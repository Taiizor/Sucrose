using CommunityToolkit.Mvvm.ComponentModel;
using System.Windows;
using System.Windows.Controls;
using System.Windows.Media;
using Wpf.Ui.Controls;
using SMC = Sucrose.Memory.Constant;
using SMMI = Sucrose.Manager.Manage.Internal;
using SMMM = Sucrose.Manager.Manage.Manager;
using SPMI = Sucrose.Portal.Manage.Internal;
using SPVCEC = Sucrose.Portal.Views.Controls.ExpanderCard;
using SSRER = Sucrose.Shared.Resources.Extension.Resources;
using TextBlock = System.Windows.Controls.TextBlock;

namespace Sucrose.Portal.ViewModels.Pages
{
    public partial class DonateSettingViewModel : ObservableObject, INavigationAware, IDisposable
    {
        [ObservableProperty]
        private List<UIElement> _Contents = new();

        private bool _isInitialized;

        public DonateSettingViewModel()
        {
            if (!_isInitialized)
            {
                InitializeViewModel();
            }
        }

        private void InitializeViewModel()
        {
            TextBlock DonateArea = new()
            {
                Foreground = SSRER.GetResource<Brush>("TextFillColorPrimaryBrush"),
                Margin = new Thickness(0, 0, 0, 0),
                FontWeight = FontWeights.Bold,
                Text = "Bağış"
            };

            Contents.Add(DonateArea);

            SPVCEC DonateMenu = new()
            {
                Margin = new Thickness(0, 10, 0, 0),
                Expandable = false
            };

            DonateMenu.Title.Text = "Bağış Menüsü";
            DonateMenu.LeftIcon.Symbol = SymbolRegular.BuildingRetailMoney24;
            DonateMenu.Description.Text = "Arayüzdeki bağış menüsünün görünürlüğü.";

            ComboBox DonateVisible = new();

            DonateVisible.SelectionChanged += (s, e) => DonateVisibleSelected(DonateVisible.SelectedIndex);

            DonateVisible.Items.Add("Görünür");
            DonateVisible.Items.Add("Görünmez");

            DonateVisible.SelectedIndex = SMMM.DonateVisible ? 0 : 1;

            DonateMenu.HeaderFrame = DonateVisible;

            Contents.Add(DonateMenu);

            TextBlock SupportArea = new()
            {
                Foreground = SSRER.GetResource<Brush>("TextFillColorPrimaryBrush"),
                Margin = new Thickness(0, 10, 0, 0),
                FontWeight = FontWeights.Bold,
                Text = "Destek"
            };

            Contents.Add(SupportArea);

            SPVCEC Advertising = new()
            {
                Margin = new Thickness(0, 10, 0, 0),
                IsExpand = true
            };

            Advertising.Title.Text = "Reklamlı Destek";
            Advertising.LeftIcon.Symbol = SymbolRegular.ReceiptMoney24;
            Advertising.Description.Text = "Reklam izleyerek uygulamanın geliştirilmesine katkıda bulun.";

            ToggleSwitch AdvertisingState = new()
            {
                IsChecked = SMMM.AdvertisingState
            };

            AdvertisingState.Checked += (s, e) => AdvertisingStateChecked(true);
            AdvertisingState.Unchecked += (s, e) => AdvertisingStateChecked(false);

            Advertising.HeaderFrame = AdvertisingState;

            StackPanel AdvertisingContent = new();

            StackPanel AdvertisingCustomContent = new()
            {
                Orientation = Orientation.Horizontal
            };

            TextBlock AdvertisingDelayText = new()
            {
                Foreground = SSRER.GetResource<Brush>("TextFillColorPrimaryBrush"),
                VerticalAlignment = VerticalAlignment.Center,
                Margin = new Thickness(0, 0, 10, 0),
                Text = "Reklam Süresi (Dakika):",
                FontWeight = FontWeights.SemiBold
            };

            NumberBox AdvertisingDelay = new()
            {
                Value = SMMM.AdvertisingDelay,
                ClearButtonEnabled = false,
                MaxLength = 3,
                Maximum = 720,
                Minimum = 30
            };

            AdvertisingDelay.ValueChanged += (s, e) => AdvertisingDelayChanged(AdvertisingDelay.Value);

            TextBlock AdvertisingHint = new()
            {
                Text = "İpucu: Reklamlar size en az seviyede rahatsızlık verecek şekilde gösterilecek.",
                Foreground = SSRER.GetResource<Brush>("TextFillColorSecondaryBrush"),
                Margin = new Thickness(0, 10, 0, 0),
                FontWeight = FontWeights.SemiBold
            };

            AdvertisingCustomContent.Children.Add(AdvertisingDelayText);
            AdvertisingCustomContent.Children.Add(AdvertisingDelay);

            AdvertisingContent.Children.Add(AdvertisingCustomContent);
            AdvertisingContent.Children.Add(AdvertisingHint);

            Advertising.FooterCard = AdvertisingContent;

            Contents.Add(Advertising);

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

        private void DonateVisibleSelected(int Index)
        {
            if (Index != (SMMM.DonateVisible ? 0 : 1))
            {
                bool State = Index == 0;
                Visibility Visible = State ? Visibility.Visible : Visibility.Collapsed;

                SMMI.DonateManager.SetSetting(SMC.DonateVisible, State);

                SPMI.DonateService.DonateVisibility = Visible;
            }
        }

        private void AdvertisingStateChecked(bool State)
        {
            SMMI.DonateManager.SetSetting(SMC.AdvertisingState, State);
        }

        private void AdvertisingDelayChanged(double? Value)
        {
            int NewValue = Convert.ToInt32(Value);

            if (NewValue != SMMM.AdvertisingDelay)
            {
                SMMI.DonateManager.SetSetting(SMC.AdvertisingDelay, NewValue);
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