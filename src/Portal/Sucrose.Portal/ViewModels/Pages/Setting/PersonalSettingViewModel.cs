using CommunityToolkit.Mvvm.ComponentModel;
using System.Windows;
using System.Windows.Controls;
using System.Windows.Media;
using Wpf.Ui.Controls;
using SMC = Sucrose.Memory.Constant;
using SMMI = Sucrose.Manager.Manage.Internal;
using SMMM = Sucrose.Manager.Manage.Manager;
using SPVCEC = Sucrose.Portal.Views.Controls.ExpanderCard;
using SSRER = Sucrose.Shared.Resources.Extension.Resources;
using TextBlock = System.Windows.Controls.TextBlock;

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
            TextBlock StoreArea = new()
            {
                Foreground = SSRER.GetResource<Brush>("TextFillColorPrimaryBrush"),
                Text = SSRER.GetValue("Portal", "Area", "Store"),
                Margin = new Thickness(0, 0, 0, 0),
                FontWeight = FontWeights.Bold
            };

            Contents.Add(StoreArea);

            SPVCEC Duration = new()
            {
                Margin = new Thickness(0, 10, 0, 0),
                Expandable = false
            };

            Duration.Title.Text = "Mağaza Yenileme";
            Duration.LeftIcon.Symbol = SymbolRegular.ClockAlarm24;
            Duration.Description.Text = "Mağazada listelenen temaların ne kadar süreyle yenilenip yenilenmeyeceği.";

            NumberBox StoreDuration = new()
            {
                ClearButtonEnabled = false,
                Value = SMMM.StoreDuration,
                MaxDecimalPlaces = 0,
                MaxLength = 2,
                Maximum = 24,
                Minimum = 1
            };

            StoreDuration.ValueChanged += (s, e) => StoreDurationChanged(StoreDuration.Value);

            Duration.HeaderFrame = StoreDuration;

            Contents.Add(Duration);

            SPVCEC Start = new()
            {
                Margin = new Thickness(0, 10, 0, 0),
                Expandable = false
            };

            Start.Title.Text = "Otomatik Başlatma";
            Start.LeftIcon.Symbol = SymbolRegular.PictureInPictureEnter24;
            Start.Description.Text = "Mağazadan indirilen temanın otomatik olarak duvar kağıdı olarak ayarlanıp ayarlanmayacağı.";

            ToggleSwitch StartState = new()
            {
                IsChecked = SMMM.Start
            };

            StartState.Checked += (s, e) => StartStateChecked(true);
            StartState.Unchecked += (s, e) => StartStateChecked(false);

            Start.HeaderFrame = StartState;

            Contents.Add(Start);

            SPVCEC Adult = new()
            {
                Margin = new Thickness(0, 10, 0, 0),
                Expandable = false
            };

            Adult.Title.Text = "Güvenli Olmayan İçerikler";
            Adult.LeftIcon.Symbol = SymbolRegular.ContentSettings24;
            Adult.Description.Text = "Mağazada güvenli değil (NSFW) olarak işaretlenmiş içeriklerin gösterilip gösterilmeyeceği.";

            ToggleSwitch AdultState = new()
            {
                IsChecked = SMMM.Adult
            };

            AdultState.Checked += (s, e) => AdultStateChecked(true);
            AdultState.Unchecked += (s, e) => AdultStateChecked(false);

            Adult.HeaderFrame = AdultState;

            Contents.Add(Adult);

            TextBlock LibraryArea = new()
            {
                Foreground = SSRER.GetResource<Brush>("TextFillColorPrimaryBrush"),
                Text = SSRER.GetValue("Portal", "Area", "Library"),
                Margin = new Thickness(0, 10, 0, 0),
                FontWeight = FontWeights.Bold
            };

            Contents.Add(LibraryArea);

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

            SPVCEC Delete = new()
            {
                Margin = new Thickness(0, 10, 0, 0),
                IsExpand = true
            };

            Delete.Title.Text = "Bozuk Temalar";
            Delete.LeftIcon.Symbol = SymbolRegular.ImageProhibited24;
            Delete.Description.Text = "Kütüphanedeki bozuk ya da eksik temaların otomatik silinip silinmeyeceği.";

            ToggleSwitch DeleteState = new()
            {
                IsChecked = SMMM.LibraryDelete
            };

            DeleteState.Checked += (s, e) => DeleteStateChecked(true);
            DeleteState.Unchecked += (s, e) => DeleteStateChecked(false);

            Delete.HeaderFrame = DeleteState;

            TextBlock DeleteHint = new()
            {
                Text = "Not: Dikkatli olunması gerekir. Aksi taktirde veri kayıplarına yol açabilir",
                Foreground = SSRER.GetResource<Brush>("TextFillColorSecondaryBrush"),
                Margin = new Thickness(0, 0, 0, 0),
                FontWeight = FontWeights.SemiBold
            };

            Delete.FooterCard = DeleteHint;

            Contents.Add(Delete);

            TextBlock AppearanceBehaviorArea = new()
            {
                Foreground = SSRER.GetResource<Brush>("TextFillColorPrimaryBrush"),
                Text = SSRER.GetValue("Portal", "Area", "AppearanceBehavior"),
                Margin = new Thickness(0, 10, 0, 0),
                FontWeight = FontWeights.Bold
            };

            Contents.Add(AppearanceBehaviorArea);

            SPVCEC Theme = new()
            {
                Margin = new Thickness(0, 10, 0, 0)
            };

            Theme.Title.Text = "Tema Özelleştirme";
            Theme.LeftIcon.Symbol = SymbolRegular.DrawText24;
            Theme.Description.Text = "Görüntülenen temaların ismini veya açıklamasını özelleştirin.";

            StackPanel ThemeContent = new();

            StackPanel ThemeTitleContent = new()
            {
                Orientation = Orientation.Horizontal
            };

            TextBlock TitleLengthText = new()
            {
                Foreground = SSRER.GetResource<Brush>("TextFillColorPrimaryBrush"),
                VerticalAlignment = VerticalAlignment.Center,
                Margin = new Thickness(0, 0, 10, 0),
                Text = "Başlık Uzunluğu (Karakter):",
                FontWeight = FontWeights.SemiBold
            };

            NumberBox TitleLength = new()
            {
                ClearButtonEnabled = false,
                Value = SMMM.TitleLength,
                MaxDecimalPlaces = 0,
                MaxLength = 3,
                Maximum = 100,
                Minimum = 10
            };

            TitleLength.ValueChanged += (s, e) => TitleLengthChanged(TitleLength.Value);

            StackPanel ThemeDescriptionContent = new()
            {
                Orientation = Orientation.Horizontal,
                Margin = new Thickness(0, 10, 0, 0)
            };

            TextBlock DescriptionLengthText = new()
            {
                Foreground = SSRER.GetResource<Brush>("TextFillColorPrimaryBrush"),
                VerticalAlignment = VerticalAlignment.Center,
                Margin = new Thickness(0, 0, 10, 0),
                Text = "Açıklama Uzunluğu (Karakter):",
                FontWeight = FontWeights.SemiBold
            };

            NumberBox DescriptionLength = new()
            {
                Value = SMMM.DescriptionLength,
                ClearButtonEnabled = false,
                MaxDecimalPlaces = 0,
                MaxLength = 3,
                Maximum = 100,
                Minimum = 10
            };

            DescriptionLength.ValueChanged += (s, e) => DescriptionLengthChanged(DescriptionLength.Value);

            ThemeTitleContent.Children.Add(TitleLengthText);
            ThemeTitleContent.Children.Add(TitleLength);

            ThemeDescriptionContent.Children.Add(DescriptionLengthText);
            ThemeDescriptionContent.Children.Add(DescriptionLength);

            ThemeContent.Children.Add(ThemeTitleContent);
            ThemeContent.Children.Add(ThemeDescriptionContent);

            Theme.FooterCard = ThemeContent;

            Contents.Add(Theme);

            SPVCEC Adaptive = new()
            {
                Margin = new Thickness(0, 10, 0, 0)
            };

            Adaptive.Title.Text = "Uyarlanabilir Düzen";
            Adaptive.LeftIcon.Symbol = SymbolRegular.BroadActivityFeed24;
            Adaptive.Description.Text = "Görüntülenen temaların düzenini veya mesafesini özelleştirin.";

            StackPanel AdaptiveContent = new();

            StackPanel AdaptiveMarginContent = new()
            {
                Orientation = Orientation.Horizontal
            };

            TextBlock AdaptiveMarginText = new()
            {
                Foreground = SSRER.GetResource<Brush>("TextFillColorPrimaryBrush"),
                VerticalAlignment = VerticalAlignment.Center,
                Margin = new Thickness(0, 0, 10, 0),
                Text = "Boşluk Mesafesi (Piksel):",
                FontWeight = FontWeights.SemiBold
            };

            NumberBox AdaptiveMargin = new()
            {
                Value = SMMM.AdaptiveMargin,
                ClearButtonEnabled = false,
                MaxDecimalPlaces = 0,
                MaxLength = 2,
                Maximum = 25,
                Minimum = 5
            };

            AdaptiveMargin.ValueChanged += (s, e) => AdaptiveMarginChanged(AdaptiveMargin.Value);

            StackPanel AdaptiveLayoutContent = new()
            {
                Orientation = Orientation.Horizontal,
                Margin = new Thickness(0, 10, 0, 0)
            };

            TextBlock AdaptiveLayoutText = new()
            {
                Foreground = SSRER.GetResource<Brush>("TextFillColorPrimaryBrush"),
                VerticalAlignment = VerticalAlignment.Center,
                Margin = new Thickness(0, 0, 10, 0),
                FontWeight = FontWeights.SemiBold,
                Text = "Düzen Sırası (Adet):"
            };

            NumberBox AdaptiveLayout = new()
            {
                Value = SMMM.AdaptiveLayout,
                ClearButtonEnabled = false,
                MaxDecimalPlaces = 0,
                MaxLength = 3,
                Maximum = 100,
                Minimum = 0
            };

            AdaptiveLayout.ValueChanged += (s, e) => AdaptiveLayoutChanged(AdaptiveLayout.Value);

            AdaptiveMarginContent.Children.Add(AdaptiveMarginText);
            AdaptiveMarginContent.Children.Add(AdaptiveMargin);

            AdaptiveLayoutContent.Children.Add(AdaptiveLayoutText);
            AdaptiveLayoutContent.Children.Add(AdaptiveLayout);

            AdaptiveContent.Children.Add(AdaptiveMarginContent);
            AdaptiveContent.Children.Add(AdaptiveLayoutContent);

            Adaptive.FooterCard = AdaptiveContent;

            Contents.Add(Adaptive);

            SPVCEC Store = new()
            {
                Margin = new Thickness(0, 10, 0, 0),
                Expandable = false
            };

            Store.Title.Text = "Mağaza Sayfalama";
            Store.LeftIcon.Symbol = SymbolRegular.DualScreenPagination24;
            Store.Description.Text = "Mağazadaki her bir sayfa için kaç tane tema olacağını belirleyin.";

            NumberBox StorePagination = new()
            {
                Value = SMMM.StorePagination,
                ClearButtonEnabled = false,
                MaxDecimalPlaces = 0,
                MaxLength = 3,
                Maximum = 100,
                Minimum = 1
            };

            StorePagination.ValueChanged += (s, e) => StorePaginationChanged(StorePagination.Value);

            Store.HeaderFrame = StorePagination;

            Contents.Add(Store);

            SPVCEC Library = new()
            {
                Margin = new Thickness(0, 10, 0, 0),
                Expandable = false
            };

            Library.Title.Text = "Kütüphane Sayfalama";
            Library.LeftIcon.Symbol = SymbolRegular.DualScreenPagination24;
            Library.Description.Text = "Kütüphanenizdeki her bir sayfa için kaç tane tema olacağını belirleyin.";

            NumberBox LibraryPagination = new()
            {
                Value = SMMM.LibraryPagination,
                ClearButtonEnabled = false,
                MaxDecimalPlaces = 0,
                MaxLength = 3,
                Maximum = 100,
                Minimum = 1
            };

            LibraryPagination.ValueChanged += (s, e) => LibraryPaginationChanged(LibraryPagination.Value);

            Library.HeaderFrame = LibraryPagination;

            Contents.Add(Library);

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

        private void AdultStateChecked(bool State)
        {
            SMMI.PortalSettingManager.SetSetting(SMC.Adult, State);
        }

        private void StartStateChecked(bool State)
        {
            SMMI.EngineSettingManager.SetSetting(SMC.Start, State);
        }

        private void DeleteStateChecked(bool State)
        {
            SMMI.LibrarySettingManager.SetSetting(SMC.LibraryDelete, State);
        }

        private void ConfirmStateChecked(bool State)
        {
            SMMI.LibrarySettingManager.SetSetting(SMC.LibraryConfirm, State);
        }

        private void TitleLengthChanged(double? Value)
        {
            int NewValue = Convert.ToInt32(Value);

            if (NewValue != SMMM.TitleLength)
            {
                SMMI.PortalSettingManager.SetSetting(SMC.TitleLength, NewValue);
            }
        }

        private void StoreDurationChanged(double? Value)
        {
            int NewValue = Convert.ToInt32(Value);

            if (NewValue != SMMM.StoreDuration)
            {
                SMMI.PortalSettingManager.SetSetting(SMC.StoreDuration, NewValue);
            }
        }

        private void AdaptiveMarginChanged(double? Value)
        {
            int NewValue = Convert.ToInt32(Value);

            if (NewValue != SMMM.AdaptiveMargin)
            {
                SMMI.PortalSettingManager.SetSetting(SMC.AdaptiveMargin, NewValue);
            }
        }

        private void AdaptiveLayoutChanged(double? Value)
        {
            int NewValue = Convert.ToInt32(Value);

            if (NewValue != SMMM.AdaptiveLayout)
            {
                SMMI.PortalSettingManager.SetSetting(SMC.AdaptiveLayout, NewValue);
            }
        }

        private void StorePaginationChanged(double? Value)
        {
            int NewValue = Convert.ToInt32(Value);

            if (NewValue != SMMM.StorePagination)
            {
                SMMI.PortalSettingManager.SetSetting(SMC.StorePagination, NewValue);
            }
        }

        private void LibraryPaginationChanged(double? Value)
        {
            int NewValue = Convert.ToInt32(Value);

            if (NewValue != SMMM.LibraryPagination)
            {
                SMMI.PortalSettingManager.SetSetting(SMC.LibraryPagination, NewValue);
            }
        }

        private void DescriptionLengthChanged(double? Value)
        {
            int NewValue = Convert.ToInt32(Value);

            if (NewValue != SMMM.DescriptionLength)
            {
                SMMI.PortalSettingManager.SetSetting(SMC.DescriptionLength, NewValue);
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