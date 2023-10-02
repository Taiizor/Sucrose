using System.Windows;
using System.Windows.Controls;
using System.Windows.Input;
using System.Windows.Media;
using Wpf.Ui.Controls;
using SEDST = Skylark.Enum.DuplicateScreenType;
using SEEST = Skylark.Enum.ExpandScreenType;
using SMC = Sucrose.Memory.Constant;
using SMMI = Sucrose.Manager.Manage.Internal;
using SMMM = Sucrose.Manager.Manage.Manager;
using SPMI = Sucrose.Portal.Manage.Internal;
using SPMM = Sucrose.Portal.Manage.Manager;
using SPVCDD = Sucrose.Portal.Views.Controls.Display.Duplicate;
using SPVCDE = Sucrose.Portal.Views.Controls.Display.Expand;
using SPVCDS = Sucrose.Portal.Views.Controls.Display.Screen;
using SSDEDT = Sucrose.Shared.Dependency.Enum.DisplayType;
using SSRER = Sucrose.Shared.Resources.Extension.Resources;
using SWUS = Skylark.Wing.Utility.Screene;
using TextBlock = System.Windows.Controls.TextBlock;

namespace Sucrose.Portal.Views.Controls
{
    /// <summary>
    /// DisplayPreferences.xaml etkileşim mantığı
    /// </summary>
    public partial class DisplayPreferences : ContentDialog, IDisposable
    {
        private StackPanel ExpanderDuplicateContent = new();

        private StackPanel ExpanderExpandContent = new();

        private StackPanel ExpanderCustomContent = new();

        public DisplayPreferences() : base(SPMI.ContentDialogService.GetContentPresenter())
        {
            InitializeComponent();
        }

        private async Task Test(SSDEDT Display)
        {
            switch (Display)
            {
                case SSDEDT.Expand:
                    break;
                case SSDEDT.Duplicate:
                    break;
                default:
                    break;
            }

            await Task.CompletedTask;
        }

        private async Task ScreenMonitor()
        {
            int ScreenCount = SWUS.Screens.Count();

            while (SMMM.ScreenIndex > ScreenCount - 1)
            {
                SMMI.EngineSettingManager.SetSetting(SMC.ScreenIndex, SMMM.ScreenIndex - 1);

                await Task.Delay(10);
            }

            for (int Count = 0; Count < ScreenCount; Count++)
            {
                SPVCDS Screen = new();

                if (SMMM.ScreenIndex == Count)
                {
                    Screen.Border.BorderBrush = Brushes.CornflowerBlue;
                }

                Screen.Index.Text = $"{Count + 1}";
                Screen.MouseLeftButtonDown += ScreenClicked;

                Contents.Children.Add(Screen);
            }

            Contents.InvalidateMeasure();

            await Task.CompletedTask;
        }

        private async Task ExpandMonitor()
        {
            SPVCDE Expand = new();

            Expand.Content.Text = "All Monitors";

            Contents.Children.Add(Expand);

            Contents.InvalidateMeasure();

            await Task.CompletedTask;
        }

        private async Task DuplicateMonitor()
        {
            for (int Count = 0; Count < SWUS.Screens.Count(); Count++)
            {
                SPVCDD Duplicate = new();

                Duplicate.Index.Text = $"{Count + 1}";

                Contents.Children.Add(Duplicate);
            }

            Contents.InvalidateMeasure();

            await Task.CompletedTask;
        }

        private void ScreenClicked(object sender, MouseButtonEventArgs e)
        {
            SPVCDS ScreenMonitor = sender as SPVCDS;

            foreach (UIElement Child in Contents.Children)
            {
                if (Child is SPVCDS Screen)
                {
                    if (Screen == ScreenMonitor)
                    {
                        Screen.Border.BorderBrush = Brushes.CornflowerBlue;

                        SMMI.EngineSettingManager.SetSetting(SMC.ScreenIndex, Convert.ToInt32(Screen.Index.Text) - 1);
                    }
                    else
                    {
                        Screen.Border.BorderBrush = SSRER.GetResource<Brush>("ControlAltFillColorTertiaryBrush");
                    }
                }
            }
        }

        private void ScreenChecked()
        {
            ExpanderCustomContent.Visibility = Visibility.Collapsed;
            ExpanderExpandContent.Visibility = Visibility.Collapsed;
            ExpanderDuplicateContent.Visibility = Visibility.Collapsed;
            SMMI.EngineSettingManager.SetSetting(SMC.DisplayType, SSDEDT.Screen);
        }

        private void ExpandChecked()
        {
            ExpanderCustomContent.Visibility = Visibility.Visible;
            ExpanderExpandContent.Visibility = Visibility.Visible;
            ExpanderDuplicateContent.Visibility = Visibility.Collapsed;
            SMMI.EngineSettingManager.SetSetting(SMC.DisplayType, SSDEDT.Expand);
        }

        private void DuplicateChecked()
        {
            ExpanderCustomContent.Visibility = Visibility.Visible;
            ExpanderExpandContent.Visibility = Visibility.Collapsed;
            ExpanderDuplicateContent.Visibility = Visibility.Visible;
            SMMI.EngineSettingManager.SetSetting(SMC.DisplayType, SSDEDT.Duplicate);
        }

        private void ExpandScreenTypeChecked(SEEST Type)
        {
            SMMI.EngineSettingManager.SetSetting(SMC.ExpandScreenType, Type);
        }

        private void DuplicateScreenTypeChecked(SEDST Type)
        {
            SMMI.EngineSettingManager.SetSetting(SMC.DuplicateScreenType, Type);
        }

        private async void ContentDialog_Loaded(object sender, RoutedEventArgs e)
        {
            StackPanel ExpanderContent = new();

            RadioButton Screen = new()
            {
                Content = "Sadece seçili ekran",
                GroupName = "DisplayType"
            };

            Screen.Checked += (s, e) => ScreenChecked();

            RadioButton Expand = new()
            {
                Content = "Ekranlar arasında uzat",
                GroupName = "DisplayType"
            };

            Expand.Checked += (s, e) => ExpandChecked();

            RadioButton Duplicate = new()
            {
                Content = "Aynı duvar kağıdını çoğalt",
                GroupName = "DisplayType"
            };

            Duplicate.Checked += (s, e) => DuplicateChecked();

            ExpanderContent.Children.Add(Screen);
            ExpanderContent.Children.Add(Expand);
            ExpanderContent.Children.Add(Duplicate);

            NavigationViewItemSeparator Separator = new()
            {
                Background = SSRER.GetResource<Brush>("TextFillColorSecondaryBrush"),
                Margin = new Thickness(0, 10, 0, 10)
            };

            ExpanderCustomContent.Children.Add(Separator);

            TextBlock ExpandHint = new()
            {
                Text = "Ekranlar arasında uzatma modunu seçin",
                Foreground = SSRER.GetResource<Brush>("TextFillColorSecondaryBrush"),
                HorizontalAlignment = HorizontalAlignment.Left,
                TextWrapping = TextWrapping.WrapWithOverflow,
                TextAlignment = TextAlignment.Left,
                Margin = new Thickness(0, 0, 0, 10),
                FontWeight = FontWeights.SemiBold
            };

            ExpanderExpandContent.Children.Add(ExpandHint);

            foreach (SEEST Type in Enum.GetValues(typeof(SEEST)))
            {
                RadioButton Radio = new()
                {
                    IsChecked = SPMM.ExpandScreenType == Type,
                    GroupName = "ExpandScreenType",
                    Content = $"{Type}"
                };

                Radio.Checked += (s, e) => ExpandScreenTypeChecked(Type);

                ExpanderExpandContent.Children.Add(Radio);
            }

            TextBlock DuplicateHint = new()
            {
                Text = "Aynı duvar kağıdını çoğaltma modunu seçin",
                Foreground = SSRER.GetResource<Brush>("TextFillColorSecondaryBrush"),
                HorizontalAlignment = HorizontalAlignment.Left,
                TextWrapping = TextWrapping.WrapWithOverflow,
                TextAlignment = TextAlignment.Left,
                Margin = new Thickness(0, 0, 0, 10),
                FontWeight = FontWeights.SemiBold
            };

            ExpanderDuplicateContent.Children.Add(DuplicateHint);

            foreach (SEDST Type in Enum.GetValues(typeof(SEDST)))
            {
                RadioButton Radio = new()
                {
                    IsChecked = SPMM.DuplicateScreenType == Type,
                    GroupName = "DuplicateScreenType",
                    Content = $"{Type}"
                };

                Radio.Checked += (s, e) => DuplicateScreenTypeChecked(Type);

                ExpanderDuplicateContent.Children.Add(Radio);
            }

            switch (SPMM.DisplayType)
            {
                case SSDEDT.Expand:
                    await ExpandMonitor();
                    Expand.IsChecked = true;
                    break;
                case SSDEDT.Duplicate:
                    await DuplicateMonitor();
                    Duplicate.IsChecked = true;
                    break;
                default:
                    await ScreenMonitor();
                    Screen.IsChecked = true;
                    break;
            }

            ExpanderContent.Children.Add(ExpanderCustomContent);
            ExpanderContent.Children.Add(ExpanderExpandContent);
            ExpanderContent.Children.Add(ExpanderDuplicateContent);

            Expander.FooterCard = ExpanderContent;
        }

        public void Dispose()
        {
            GC.Collect();
            GC.SuppressFinalize(this);
        }
    }
}