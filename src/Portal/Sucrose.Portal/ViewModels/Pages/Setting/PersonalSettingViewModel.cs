using CommunityToolkit.Mvvm.ComponentModel;
using System.Windows;
using System.Windows.Controls;
using System.Windows.Media;
using Wpf.Ui.Controls;
using SMMCE = Sucrose.Memory.Manage.Constant.Engine;
using SMMCL = Sucrose.Memory.Manage.Constant.Library;
using SMMCP = Sucrose.Memory.Manage.Constant.Portal;
using SMME = Sucrose.Manager.Manage.Engine;
using SMMI = Sucrose.Manager.Manage.Internal;
using SMML = Sucrose.Manager.Manage.Library;
using SMMP = Sucrose.Manager.Manage.Portal;
using SPVCEC = Sucrose.Portal.Views.Controls.ExpanderCard;
using SRER = Sucrose.Resources.Extension.Resources;
using SSDESKT = Sucrose.Shared.Dependency.Enum.SortKindType;
using SSDESMT = Sucrose.Shared.Dependency.Enum.SortModeType;
using SSDESST = Sucrose.Shared.Dependency.Enum.StoreServerType;
using SSDMMP = Sucrose.Shared.Dependency.Manage.Manager.Portal;
using SSSMI = Sucrose.Shared.Store.Manage.Internal;
using TextBlock = System.Windows.Controls.TextBlock;

namespace Sucrose.Portal.ViewModels.Pages
{
    public partial class PersonalSettingViewModel : ViewModel, IDisposable
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
                Foreground = SRER.GetResource<Brush>("TextFillColorPrimaryBrush"),
                Text = SRER.GetValue("Portal", "Area", "Store"),
                Margin = new Thickness(0, 0, 0, 0),
                FontWeight = FontWeights.Bold
            };

            Contents.Add(StoreArea);

            SPVCEC Server = new()
            {
                Margin = new Thickness(0, 10, 0, 0)
            };

            Server.LeftIcon.Symbol = SymbolRegular.ServerSurfaceMultiple16; //SymbolRegular.CloudFlow24
            Server.Title.Text = SRER.GetValue("Portal", "PersonalSettingPage", "Server");
            Server.Description.Text = SRER.GetValue("Portal", "PersonalSettingPage", "Server", "Description");

            ComboBox StoreType = new();

            StoreType.SelectionChanged += (s, e) => StoreTypeSelected(StoreType.SelectedIndex);

            foreach (SSDESST Type in Enum.GetValues(typeof(SSDESST)))
            {
                StoreType.Items.Add(new ComboBoxItem()
                {
                    Content = SRER.GetValue("Portal", "Enum", "StoreServerType", $"{Type}")
                });
            }

            StoreType.SelectedIndex = (int)SSDMMP.StoreServerType;

            Server.HeaderFrame = StoreType;

            TextBlock ServerHint = new()
            {
                Text = string.Format(SRER.GetValue("Portal", "PersonalSettingPage", "Server", "ServerHint"), SRER.GetValue("Portal", "Enum", "StoreServerType", $"{SSDESST.GitHub}"), SRER.GetValue("Portal", "Enum", "StoreServerType", $"{SSDESST.Soferity}"), SRER.GetValue("Portal", "OtherSettingPage", "Key")),
                Foreground = SRER.GetResource<Brush>("TextFillColorSecondaryBrush"),
                HorizontalAlignment = HorizontalAlignment.Left,
                TextWrapping = TextWrapping.WrapWithOverflow,
                TextAlignment = TextAlignment.Left,
                Margin = new Thickness(0, 0, 0, 0),
                FontWeight = FontWeights.SemiBold
            };

            Server.FooterCard = ServerHint;

            Contents.Add(Server);

            SPVCEC Duration = new()
            {
                Margin = new Thickness(0, 10, 0, 0),
                Expandable = false
            };

            Duration.LeftIcon.Symbol = SymbolRegular.ClockAlarm24;
            Duration.Title.Text = SRER.GetValue("Portal", "PersonalSettingPage", "Duration");
            Duration.Description.Text = SRER.GetValue("Portal", "PersonalSettingPage", "Duration", "Description");

            NumberBox StoreDuration = new()
            {
                Icon = new SymbolIcon(SymbolRegular.Clock24),
                IconPlacement = ElementPlacement.Left,
                ClearButtonEnabled = false,
                Value = SMMP.StoreDuration,
                MaxDecimalPlaces = 0,
                MaxLength = 2,
                Maximum = 24,
                Minimum = 1
            };

            StoreDuration.ValueChanged += (s, e) => StoreDurationChanged(StoreDuration.Value);

            Duration.HeaderFrame = StoreDuration;

            Contents.Add(Duration);

            SPVCEC StoreStart = new()
            {
                Margin = new Thickness(0, 10, 0, 0),
                Expandable = false
            };

            StoreStart.LeftIcon.Symbol = SymbolRegular.PictureInPictureEnter24;
            StoreStart.Title.Text = SRER.GetValue("Portal", "PersonalSettingPage", "StoreStart");
            StoreStart.Description.Text = SRER.GetValue("Portal", "PersonalSettingPage", "StoreStart", "Description");

            ToggleSwitch StoreStartState = new()
            {
                IsChecked = SMME.StoreStart
            };

            StoreStartState.Checked += (s, e) => StoreStartStateChecked(true);
            StoreStartState.Unchecked += (s, e) => StoreStartStateChecked(false);

            StoreStart.HeaderFrame = StoreStartState;

            Contents.Add(StoreStart);

            SPVCEC Adult = new()
            {
                Margin = new Thickness(0, 10, 0, 0),
                Expandable = false
            };

            Adult.LeftIcon.Symbol = SymbolRegular.ShieldGlobe24;
            Adult.Title.Text = SRER.GetValue("Portal", "PersonalSettingPage", "Adult");
            Adult.Description.Text = SRER.GetValue("Portal", "PersonalSettingPage", "Adult", "Description");

            ToggleSwitch AdultState = new()
            {
                IsChecked = SMMP.Adult
            };

            AdultState.Checked += (s, e) => AdultStateChecked(true);
            AdultState.Unchecked += (s, e) => AdultStateChecked(false);

            Adult.HeaderFrame = AdultState;

            Contents.Add(Adult);

            TextBlock LibraryArea = new()
            {
                Foreground = SRER.GetResource<Brush>("TextFillColorPrimaryBrush"),
                Text = SRER.GetValue("Portal", "Area", "Library"),
                Margin = new Thickness(0, 10, 0, 0),
                FontWeight = FontWeights.Bold
            };

            Contents.Add(LibraryArea);

            SPVCEC Confirm = new()
            {
                Margin = new Thickness(0, 10, 0, 0),
                Expandable = false
            };

            Confirm.LeftIcon.Symbol = SymbolRegular.DeleteDismiss24;
            Confirm.Title.Text = SRER.GetValue("Portal", "PersonalSettingPage", "Confirm");
            Confirm.Description.Text = SRER.GetValue("Portal", "PersonalSettingPage", "Confirm", "Description");

            ToggleSwitch ConfirmState = new()
            {
                IsChecked = SMML.LibraryConfirm
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

            Delete.LeftIcon.Symbol = SymbolRegular.ImageProhibited24;
            Delete.Title.Text = SRER.GetValue("Portal", "PersonalSettingPage", "Delete");
            Delete.Description.Text = SRER.GetValue("Portal", "PersonalSettingPage", "Delete", "Description");

            ToggleSwitch DeleteState = new()
            {
                IsChecked = SMML.LibraryDelete
            };

            DeleteState.Checked += (s, e) => DeleteStateChecked(true);
            DeleteState.Unchecked += (s, e) => DeleteStateChecked(false);

            Delete.HeaderFrame = DeleteState;

            TextBlock DeleteHint = new()
            {
                Text = SRER.GetValue("Portal", "PersonalSettingPage", "Delete", "DeleteHint"),
                Foreground = SRER.GetResource<Brush>("TextFillColorSecondaryBrush"),
                HorizontalAlignment = HorizontalAlignment.Left,
                TextWrapping = TextWrapping.WrapWithOverflow,
                TextAlignment = TextAlignment.Left,
                Margin = new Thickness(0, 0, 0, 0),
                FontWeight = FontWeights.SemiBold
            };

            Delete.FooterCard = DeleteHint;

            Contents.Add(Delete);

            SPVCEC LibraryStart = new()
            {
                Margin = new Thickness(0, 10, 0, 0),
                Expandable = false
            };

            LibraryStart.LeftIcon.Symbol = SymbolRegular.PictureInPictureEnter24;
            LibraryStart.Title.Text = SRER.GetValue("Portal", "PersonalSettingPage", "LibraryStart");
            LibraryStart.Description.Text = SRER.GetValue("Portal", "PersonalSettingPage", "LibraryStart", "Description");

            ToggleSwitch LibraryStartState = new()
            {
                IsChecked = SMME.LibraryStart
            };

            LibraryStartState.Checked += (s, e) => LibraryStartStateChecked(true);
            LibraryStartState.Unchecked += (s, e) => LibraryStartStateChecked(false);

            LibraryStart.HeaderFrame = LibraryStartState;

            Contents.Add(LibraryStart);

            TextBlock AppearanceBehaviorArea = new()
            {
                Foreground = SRER.GetResource<Brush>("TextFillColorPrimaryBrush"),
                Text = SRER.GetValue("Portal", "Area", "AppearanceBehavior"),
                Margin = new Thickness(0, 10, 0, 0),
                FontWeight = FontWeights.Bold
            };

            Contents.Add(AppearanceBehaviorArea);

            SPVCEC Preview = new()
            {
                Margin = new Thickness(0, 10, 0, 0)
            };

            Preview.LeftIcon.Symbol = SymbolRegular.ResizeVideo24;
            Preview.Title.Text = SRER.GetValue("Portal", "PersonalSettingPage", "Preview");
            Preview.Description.Text = SRER.GetValue("Portal", "PersonalSettingPage", "Preview", "Description");

            StackPanel PreviewContent = new();

            StackPanel PreviewStoreContent = new()
            {
                Orientation = Orientation.Horizontal
            };

            TextBlock StorePreviewText = new()
            {
                Text = SRER.GetValue("Portal", "PersonalSettingPage", "Preview", "StorePreview"),
                Foreground = SRER.GetResource<Brush>("TextFillColorPrimaryBrush"),
                VerticalAlignment = VerticalAlignment.Center,
                Margin = new Thickness(0, 0, 10, 0),
                FontWeight = FontWeights.SemiBold
            };

            ToggleSwitch StorePreview = new()
            {
                IsChecked = SMMP.StorePreview
            };

            StorePreview.Checked += (s, e) => StorePreviewChecked(true);
            StorePreview.Unchecked += (s, e) => StorePreviewChecked(false);

            TextBlock StorePreviewHideText = new()
            {
                Text = SRER.GetValue("Portal", "PersonalSettingPage", "Preview", "StorePreviewHide"),
                Foreground = SRER.GetResource<Brush>("TextFillColorPrimaryBrush"),
                VerticalAlignment = VerticalAlignment.Center,
                Margin = new Thickness(20, 0, 10, 0),
                FontWeight = FontWeights.SemiBold
            };

            ToggleSwitch StorePreviewHide = new()
            {
                IsChecked = SMMP.StorePreviewHide
            };

            StorePreviewHide.Checked += (s, e) => StorePreviewHideChecked(true);
            StorePreviewHide.Unchecked += (s, e) => StorePreviewHideChecked(false);

            StackPanel PreviewLibraryContent = new()
            {
                Orientation = Orientation.Horizontal,
                Margin = new Thickness(0, 10, 0, 0)
            };

            TextBlock LibraryPreviewText = new()
            {
                Text = SRER.GetValue("Portal", "PersonalSettingPage", "Preview", "LibraryPreview"),
                Foreground = SRER.GetResource<Brush>("TextFillColorPrimaryBrush"),
                VerticalAlignment = VerticalAlignment.Center,
                Margin = new Thickness(0, 0, 10, 0),
                FontWeight = FontWeights.SemiBold
            };

            ToggleSwitch LibraryPreview = new()
            {
                IsChecked = SMMP.LibraryPreview
            };

            LibraryPreview.Checked += (s, e) => LibraryPreviewChecked(true);
            LibraryPreview.Unchecked += (s, e) => LibraryPreviewChecked(false);

            TextBlock LibraryPreviewHideText = new()
            {
                Text = SRER.GetValue("Portal", "PersonalSettingPage", "Preview", "LibraryPreviewHide"),
                Foreground = SRER.GetResource<Brush>("TextFillColorPrimaryBrush"),
                VerticalAlignment = VerticalAlignment.Center,
                Margin = new Thickness(20, 0, 10, 0),
                FontWeight = FontWeights.SemiBold
            };

            ToggleSwitch LibraryPreviewHide = new()
            {
                IsChecked = SMMP.LibraryPreviewHide
            };

            LibraryPreviewHide.Checked += (s, e) => LibraryPreviewHideChecked(true);
            LibraryPreviewHide.Unchecked += (s, e) => LibraryPreviewHideChecked(false);

            PreviewStoreContent.Children.Add(StorePreviewText);
            PreviewStoreContent.Children.Add(StorePreview);
            PreviewStoreContent.Children.Add(StorePreviewHideText);
            PreviewStoreContent.Children.Add(StorePreviewHide);

            PreviewLibraryContent.Children.Add(LibraryPreviewText);
            PreviewLibraryContent.Children.Add(LibraryPreview);
            PreviewLibraryContent.Children.Add(LibraryPreviewHideText);
            PreviewLibraryContent.Children.Add(LibraryPreviewHide);

            PreviewContent.Children.Add(PreviewStoreContent);
            PreviewContent.Children.Add(PreviewLibraryContent);

            Preview.FooterCard = PreviewContent;

            Contents.Add(Preview);

            SPVCEC Adaptive = new()
            {
                Margin = new Thickness(0, 10, 0, 0)
            };

            Adaptive.LeftIcon.Symbol = SymbolRegular.BroadActivityFeed24;
            Adaptive.Title.Text = SRER.GetValue("Portal", "PersonalSettingPage", "Adaptive");
            Adaptive.Description.Text = SRER.GetValue("Portal", "PersonalSettingPage", "Adaptive", "Description");

            StackPanel AdaptiveContent = new();

            StackPanel AdaptiveMarginContent = new()
            {
                Orientation = Orientation.Horizontal
            };

            TextBlock AdaptiveMarginText = new()
            {
                Text = SRER.GetValue("Portal", "PersonalSettingPage", "Adaptive", "AdaptiveMargin"),
                Foreground = SRER.GetResource<Brush>("TextFillColorPrimaryBrush"),
                VerticalAlignment = VerticalAlignment.Center,
                Margin = new Thickness(0, 0, 10, 0),
                FontWeight = FontWeights.SemiBold
            };

            NumberBox AdaptiveMargin = new()
            {
                Icon = new SymbolIcon(SymbolRegular.DocumentMargins24),
                IconPlacement = ElementPlacement.Left,
                Value = SMMP.AdaptiveMargin,
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
                Text = SRER.GetValue("Portal", "PersonalSettingPage", "Adaptive", "AdaptiveLayout"),
                Foreground = SRER.GetResource<Brush>("TextFillColorPrimaryBrush"),
                VerticalAlignment = VerticalAlignment.Center,
                Margin = new Thickness(0, 0, 10, 0),
                FontWeight = FontWeights.SemiBold
            };

            NumberBox AdaptiveLayout = new()
            {
                Icon = new SymbolIcon(SymbolRegular.NumberRow24),
                IconPlacement = ElementPlacement.Left,
                Value = SMMP.AdaptiveLayout,
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

            SPVCEC Sort = new()
            {
                Margin = new Thickness(0, 10, 0, 0),
                Expandable = true,
                IsExpand = true
            };

            Sort.LeftIcon.Symbol = SymbolRegular.ArrowSort24;
            Sort.Title.Text = SRER.GetValue("Portal", "PersonalSettingPage", "Sort");
            Sort.Description.Text = SRER.GetValue("Portal", "PersonalSettingPage", "Sort", "Description");

            StackPanel SortContent = new()
            {
                Orientation = Orientation.Horizontal,
                Margin = new Thickness(0, 10, 0, 0)
            };

            TextBlock SortModeText = new()
            {
                Text = SRER.GetValue("Portal", "PersonalSettingPage", "Sort", "SortMode"),
                Foreground = SRER.GetResource<Brush>("TextFillColorPrimaryBrush"),
                VerticalAlignment = VerticalAlignment.Center,
                Margin = new Thickness(0, 0, 10, 0),
                FontWeight = FontWeights.SemiBold
            };

            ComboBox SortMode = new();

            SortMode.SelectionChanged += (s, e) => SortModeSelected(SortMode.SelectedIndex);

            foreach (SSDESMT Type in Enum.GetValues(typeof(SSDESMT)))
            {
                SortMode.Items.Add(new ComboBoxItem()
                {
                    Content = SRER.GetValue("Portal", "Enum", "SortModeType", $"{Type}")
                });
            }

            SortMode.SelectedIndex = (int)SSDMMP.LibrarySortMode;

            TextBlock SortKindText = new()
            {
                Text = SRER.GetValue("Portal", "PersonalSettingPage", "Sort", "SortKind"),
                Foreground = SRER.GetResource<Brush>("TextFillColorPrimaryBrush"),
                VerticalAlignment = VerticalAlignment.Center,
                Margin = new Thickness(20, 0, 10, 0),
                FontWeight = FontWeights.SemiBold
            };

            ComboBox SortKind = new();

            SortKind.SelectionChanged += (s, e) => SortKindSelected(SortKind.SelectedIndex);

            foreach (SSDESKT Type in Enum.GetValues(typeof(SSDESKT)))
            {
                SortKind.Items.Add(new ComboBoxItem()
                {
                    Content = SRER.GetValue("Portal", "Enum", "SortKindType", $"{Type}")
                });
            }

            SortKind.SelectedIndex = (int)SSDMMP.LibrarySortKind;

            SortContent.Children.Add(SortModeText);
            SortContent.Children.Add(SortMode);
            SortContent.Children.Add(SortKindText);
            SortContent.Children.Add(SortKind);

            Sort.FooterCard = SortContent;

            Contents.Add(Sort);

            SPVCEC Store = new()
            {
                Margin = new Thickness(0, 10, 0, 0),
                Expandable = false
            };

            Store.LeftIcon.Symbol = SymbolRegular.DualScreenPagination24;
            Store.Title.Text = SRER.GetValue("Portal", "PersonalSettingPage", "Store");
            Store.Description.Text = SRER.GetValue("Portal", "PersonalSettingPage", "Store", "Description");

            NumberBox StorePagination = new()
            {
                Icon = new SymbolIcon(SymbolRegular.TextWordCount24),
                IconPlacement = ElementPlacement.Left,
                Value = SMMP.StorePagination,
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

            Library.LeftIcon.Symbol = SymbolRegular.DualScreenPagination24;
            Library.Title.Text = SRER.GetValue("Portal", "PersonalSettingPage", "Library");
            Library.Description.Text = SRER.GetValue("Portal", "PersonalSettingPage", "Library", "Description");

            NumberBox LibraryPagination = new()
            {
                Icon = new SymbolIcon(SymbolRegular.TextWordCount24),
                IconPlacement = ElementPlacement.Left,
                Value = SMMP.LibraryPagination,
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

        private void SortKindSelected(int Index)
        {
            SSDESKT NewKind = (SSDESKT)Index;

            if (NewKind != SSDMMP.LibrarySortKind)
            {
                SMMI.PortalSettingManager.SetSetting(SMMCP.LibrarySortKind, NewKind);
            }
        }

        private void SortModeSelected(int Index)
        {
            SSDESMT NewMode = (SSDESMT)Index;

            if (NewMode != SSDMMP.LibrarySortMode)
            {
                SMMI.PortalSettingManager.SetSetting(SMMCP.LibrarySortMode, NewMode);
            }
        }

        private void StoreTypeSelected(int Index)
        {
            SSDESST NewStore = (SSDESST)Index;

            if (NewStore != SSDMMP.StoreServerType)
            {
                SSSMI.State = true;

                SMMI.PortalSettingManager.SetSetting(SMMCP.StoreServerType, NewStore);
            }
        }

        private void AdultStateChecked(bool State)
        {
            SMMI.PortalSettingManager.SetSetting(SMMCP.Adult, State);
        }

        private void DeleteStateChecked(bool State)
        {
            SMMI.LibrarySettingManager.SetSetting(SMMCL.LibraryDelete, State);
        }

        private void ConfirmStateChecked(bool State)
        {
            SMMI.LibrarySettingManager.SetSetting(SMMCL.LibraryConfirm, State);
        }

        private void StorePreviewChecked(bool State)
        {
            SMMI.PortalSettingManager.SetSetting(SMMCP.StorePreview, State);
        }

        private void LibraryPreviewChecked(bool State)
        {
            SMMI.PortalSettingManager.SetSetting(SMMCP.LibraryPreview, State);
        }

        private void StoreStartStateChecked(bool State)
        {
            SMMI.EngineSettingManager.SetSetting(SMMCE.StoreStart, State);
        }

        private void StoreDurationChanged(double? Value)
        {
            int NewValue = Convert.ToInt32(Value);

            if (NewValue != SMMP.StoreDuration)
            {
                SMMI.PortalSettingManager.SetSetting(SMMCP.StoreDuration, NewValue);
            }
        }

        private void StorePreviewHideChecked(bool State)
        {
            SMMI.PortalSettingManager.SetSetting(SMMCP.StorePreviewHide, State);
        }

        private void LibraryStartStateChecked(bool State)
        {
            SMMI.EngineSettingManager.SetSetting(SMMCE.LibraryStart, State);
        }

        private void AdaptiveMarginChanged(double? Value)
        {
            int NewValue = Convert.ToInt32(Value);

            if (NewValue != SMMP.AdaptiveMargin)
            {
                SMMI.PortalSettingManager.SetSetting(SMMCP.AdaptiveMargin, NewValue);
            }
        }

        private void AdaptiveLayoutChanged(double? Value)
        {
            int NewValue = Convert.ToInt32(Value);

            if (NewValue != SMMP.AdaptiveLayout)
            {
                SMMI.PortalSettingManager.SetSetting(SMMCP.AdaptiveLayout, NewValue);
            }
        }

        private void LibraryPreviewHideChecked(bool State)
        {
            SMMI.PortalSettingManager.SetSetting(SMMCP.LibraryPreviewHide, State);
        }

        private void StorePaginationChanged(double? Value)
        {
            int NewValue = Convert.ToInt32(Value);

            if (NewValue != SMMP.StorePagination)
            {
                SMMI.PortalSettingManager.SetSetting(SMMCP.StorePagination, NewValue);
            }
        }

        private void LibraryPaginationChanged(double? Value)
        {
            int NewValue = Convert.ToInt32(Value);

            if (NewValue != SMMP.LibraryPagination)
            {
                SMMI.PortalSettingManager.SetSetting(SMMCP.LibraryPagination, NewValue);
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