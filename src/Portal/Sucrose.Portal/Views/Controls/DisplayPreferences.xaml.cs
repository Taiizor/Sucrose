using System.Windows;
using System.Windows.Controls;
using System.Windows.Input;
using System.Windows.Media;
using Wpf.Ui.Controls;
using SEDEST = Skylark.Enum.DuplicateScreenType;
using SEDYST = Skylark.Enum.DisplayScreenType;
using SEEST = Skylark.Enum.ExpandScreenType;
using SMC = Sucrose.Memory.Constant;
using SMMI = Sucrose.Manager.Manage.Internal;
using SMMM = Sucrose.Manager.Manage.Manager;
using SPVCDD = Sucrose.Portal.Views.Controls.Display.Duplicate;
using SPVCDE = Sucrose.Portal.Views.Controls.Display.Expand;
using SPVCDS = Sucrose.Portal.Views.Controls.Display.Screen;
using SRER = Sucrose.Resources.Extension.Resources;
using SSLHK = Sucrose.Shared.Live.Helper.Kill;
using SSLHR = Sucrose.Shared.Live.Helper.Run;
using SSSHL = Sucrose.Shared.Space.Helper.Live;
using SSSHP = Sucrose.Shared.Space.Helper.Processor;
using SSSMI = Sucrose.Shared.Space.Manage.Internal;
using SWUS = Skylark.Wing.Utility.Screene;
using TextBlock = System.Windows.Controls.TextBlock;
using SMME = Sucrose.Manager.Manage.Engine;
using SMMCE = Sucrose.Memory.Manage.Constant.Engine;
using SMMMCE = Sucrose.Memory.Manage.Constant.Engine;

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

        public DisplayPreferences(ContentPresenter? contentPresenter) : base(contentPresenter)
        {
            InitializeComponent();
        }

        private void Restart()
        {
            if ((!SMMM.ClosePerformance && !SMMM.PausePerformance) || !SSSHP.Work(SSSMI.Backgroundog))
            {
                if (SSSHL.Run())
                {
                    SSLHK.Stop();
                }

                SSLHR.Start();
            }
        }

        private async Task ScreenMonitor()
        {
            Contents.Children.Clear();

            SWUS.Initialize();

            int ScreenCount = SWUS.Screens.Count();

            if (SMME.ScreenIndex > ScreenCount - 1)
            {
                SMMI.EngineSettingManager.SetSetting(SMMCE.ScreenIndex, ScreenCount - 1);
            }

            for (int Count = 0; Count < ScreenCount; Count++)
            {
                SPVCDS Screen = new();

                if (SMME.ScreenIndex == Count)
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
            Contents.Children.Clear();

            SPVCDE Expand = new();

            Expand.Title.Text = SRER.GetValue("Portal", "DisplayPreferences", "Expand", "Monitor");

            Contents.Children.Add(Expand);

            Contents.InvalidateMeasure();

            await Task.CompletedTask;
        }

        private async Task DuplicateMonitor()
        {
            Contents.Children.Clear();

            SWUS.Initialize();

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

                        SMMI.EngineSettingManager.SetSetting(SMMCE.ScreenIndex, Convert.ToInt32(Screen.Index.Text) - 1);
                    }
                    else
                    {
                        Screen.Border.BorderBrush = SRER.GetResource<Brush>("ControlAltFillColorTertiaryBrush");
                    }
                }
            }

            Restart();
        }

        private async void ScreenChecked()
        {
            await ScreenMonitor();

            ExpanderCustomContent.Visibility = Visibility.Collapsed;
            ExpanderExpandContent.Visibility = Visibility.Collapsed;
            ExpanderDuplicateContent.Visibility = Visibility.Collapsed;

            if (SMME.DisplayScreenType != SEDYST.PerDisplay)
            {
                SMMI.EngineSettingManager.SetSetting(SMMCE.DisplayScreenType, SEDYST.PerDisplay);

                Restart();
            }
        }

        private async void ExpandChecked()
        {
            await ExpandMonitor();

            ExpanderCustomContent.Visibility = Visibility.Visible;
            ExpanderExpandContent.Visibility = Visibility.Visible;
            ExpanderDuplicateContent.Visibility = Visibility.Collapsed;

            if (SMME.DisplayScreenType != SEDYST.SpanAcross)
            {
                SMMI.EngineSettingManager.SetSetting(SMMCE.DisplayScreenType, SEDYST.SpanAcross);

                Restart();
            }
        }

        private async void DuplicateChecked()
        {
            await DuplicateMonitor();

            ExpanderCustomContent.Visibility = Visibility.Visible;
            ExpanderExpandContent.Visibility = Visibility.Collapsed;
            ExpanderDuplicateContent.Visibility = Visibility.Visible;

            if (SMME.DisplayScreenType != SEDYST.SameDuplicate)
            {
                SMMI.EngineSettingManager.SetSetting(SMMCE.DisplayScreenType, SEDYST.SameDuplicate);

                Restart();
            }
        }

        private void ExpandScreenTypeChecked(SEEST Type)
        {
            if (SMME.ExpandScreenType != Type)
            {
                SMMI.EngineSettingManager.SetSetting(SMMCE.ExpandScreenType, Type);

                Restart();
            }
        }

        private void DuplicateScreenTypeChecked(SEDEST Type)
        {
            if (SMME.DuplicateScreenType != Type)
            {
                SMMI.EngineSettingManager.SetSetting(SMMCE.DuplicateScreenType, Type);

                Restart();
            }
        }

        private async void ContentDialog_Loaded(object sender, RoutedEventArgs e)
        {
            Expander.Title.Text = SRER.GetValue("Portal", "DisplayPreferences", "Expander");
            Expander.Description.Text = SRER.GetValue("Portal", "DisplayPreferences", "Expander", "Description");

            StackPanel ExpanderContent = new();

            RadioButton Screen = new()
            {
                Content = SRER.GetValue("Portal", "DisplayPreferences", "Screen"),
                GroupName = "DisplayType"
            };

            Screen.Checked += (s, e) => ScreenChecked();

            RadioButton Expand = new()
            {
                Content = SRER.GetValue("Portal", "DisplayPreferences", "Expand"),
                GroupName = "DisplayType"
            };

            Expand.Checked += (s, e) => ExpandChecked();

            RadioButton Duplicate = new()
            {
                Content = SRER.GetValue("Portal", "DisplayPreferences", "Duplicate"),
                GroupName = "DisplayType",
                IsEnabled = false
            };

            Duplicate.Checked += (s, e) => DuplicateChecked();

            ExpanderContent.Children.Add(Screen);
            ExpanderContent.Children.Add(Expand);
            ExpanderContent.Children.Add(Duplicate);

            NavigationViewItemSeparator Separator = new()
            {
                Background = SRER.GetResource<Brush>("TextFillColorSecondaryBrush"),
                Margin = new Thickness(0, 10, 0, 10)
            };

            ExpanderCustomContent.Children.Add(Separator);

            TextBlock ExpandHint = new()
            {
                Text = SRER.GetValue("Portal", "DisplayPreferences", "Expand", "Hint"),
                Foreground = SRER.GetResource<Brush>("TextFillColorSecondaryBrush"),
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
                    Content = SRER.GetValue("Portal", "Enum", "ExpandScreenType", $"{Type}"),
                    IsChecked = SMME.ExpandScreenType == Type,
                    GroupName = "ExpandScreenType"
                };

                Radio.Checked += (s, e) => ExpandScreenTypeChecked(Type);

                ExpanderExpandContent.Children.Add(Radio);
            }

            TextBlock DuplicateHint = new()
            {
                Text = SRER.GetValue("Portal", "DisplayPreferences", "Duplicate", "Hint"),
                Foreground = SRER.GetResource<Brush>("TextFillColorSecondaryBrush"),
                HorizontalAlignment = HorizontalAlignment.Left,
                TextWrapping = TextWrapping.WrapWithOverflow,
                TextAlignment = TextAlignment.Left,
                Margin = new Thickness(0, 0, 0, 10),
                FontWeight = FontWeights.SemiBold
            };

            ExpanderDuplicateContent.Children.Add(DuplicateHint);

            foreach (SEDEST Type in Enum.GetValues(typeof(SEDEST)))
            {
                RadioButton Radio = new()
                {
                    Content = SRER.GetValue("Portal", "Enum", "DuplicateScreenType", $"{Type}"),
                    IsChecked = SMME.DuplicateScreenType == Type,
                    GroupName = "DuplicateScreenType"
                };

                Radio.Checked += (s, e) => DuplicateScreenTypeChecked(Type);

                ExpanderDuplicateContent.Children.Add(Radio);
            }

            switch (SMME.DisplayScreenType)
            {
                case SEDYST.SpanAcross:
                    Expand.IsChecked = true;
                    break;
                case SEDYST.SameDuplicate:
                    Duplicate.IsChecked = true;
                    break;
                default:
                    Screen.IsChecked = true;
                    break;
            }

            ExpanderContent.Children.Add(ExpanderCustomContent);
            ExpanderContent.Children.Add(ExpanderExpandContent);
            ExpanderContent.Children.Add(ExpanderDuplicateContent);

            Expander.FooterCard = ExpanderContent;

            await Task.Delay(10);

            Panel.MinHeight = 0;
        }

        public void Dispose()
        {
            GC.Collect();
            GC.SuppressFinalize(this);
        }
    }
}