using System.Windows;
using System.Windows.Controls;
using System.Windows.Input;
using System.Windows.Media;
using Wpf.Ui.Controls;
using SEDEST = Skylark.Enum.DuplicateScreenType;
using SEDYST = Skylark.Enum.DisplayScreenType;
using SEEST = Skylark.Enum.ExpandScreenType;
using SMMB = Sucrose.Manager.Manage.Backgroundog;
using SMMCE = Sucrose.Memory.Manage.Constant.Engine;
using SMME = Sucrose.Manager.Manage.Engine;
using SMMI = Sucrose.Manager.Manage.Internal;
using SPVCDD = Sucrose.Portal.Views.Controls.Display.Duplicate;
using SPVCDS = Sucrose.Portal.Views.Controls.Display.Screen;
using SRER = Sucrose.Resources.Extension.Resources;
using SSLHK = Sucrose.Shared.Live.Helper.Kill;
using SSLHR = Sucrose.Shared.Live.Helper.Run;
using SSSHL = Sucrose.Shared.Space.Helper.Live;
using SSSHP = Sucrose.Shared.Space.Helper.Processor;
using SSSMI = Sucrose.Shared.Space.Manage.Internal;
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

        public DisplayPreferences(ContentDialogHost? contentDialogHost) : base(contentDialogHost)
        {
            InitializeComponent();
        }

        private void Restart()
        {
            if ((!SMMB.ClosePerformance && !SMMB.PausePerformance) || !SSSHP.Work(SSSMI.Backgroundog))
            {
                if (SSSHL.Run())
                {
                    SSLHK.Stop();
                }

                SSLHR.Start();
            }
        }

        private async Task ExpandMonitor()
        {
            Contents.Children.Clear();

            List<(int Left, int Top, int Width, int Height)> Screens = GetScreenBounds();

            int MinX = Screens.Min(s => s.Left);
            int MaxX = Screens.Max(s => s.Left + s.Width);
            int MinY = Screens.Min(s => s.Top);
            int MaxY = Screens.Max(s => s.Top + s.Height);

            double TotalW = MaxX - MinX;
            double TotalH = MaxY - MinY;

            double CanvasW = 680;
            double CanvasH = 230;
            double Padding = 15;

            double ScaleX = (CanvasW - (2 * Padding)) / TotalW;
            double ScaleY = (CanvasH - (2 * Padding)) / TotalH;
            double Scale = Math.Min(ScaleX, ScaleY);

            double ScaledTotalW = TotalW * Scale;
            double ScaledTotalH = TotalH * Scale;

            double OffsetX = (CanvasW - ScaledTotalW) / 2.0;
            double OffsetY = (CanvasH - ScaledTotalH) / 2.0;

            for (int Count = 0; Count < Screens.Count; Count++)
            {
                SPVCDD Duplicate = new();

                Duplicate.Index.Text = $"{Count + 1}";
                Duplicate.Border.BorderBrush = Brushes.CornflowerBlue;

                (int Left, int Top, int Width, int Height) Monitor = Screens[Count];

                double Left = OffsetX + ((Monitor.Left - MinX) * Scale);
                double Top = OffsetY + ((Monitor.Top - MinY) * Scale);
                double Width = Math.Max(25, Monitor.Width * Scale);
                double Height = Math.Max(25, Monitor.Height * Scale);

                Canvas.SetLeft(Duplicate, Left);
                Canvas.SetTop(Duplicate, Top);

                Duplicate.Width = Width;
                Duplicate.Height = Height;

                Contents.Children.Add(Duplicate);
            }

            Border BoundingOutline = new()
            {
                BorderBrush = Brushes.CornflowerBlue,
                BorderThickness = new Thickness(2),
                CornerRadius = new CornerRadius(8),
                Background = new SolidColorBrush(Color.FromArgb(20, 100, 149, 237)),
                IsHitTestVisible = false
            };

            Canvas.SetLeft(BoundingOutline, OffsetX - 4);
            Canvas.SetTop(BoundingOutline, OffsetY - 4);

            BoundingOutline.Width = ScaledTotalW + 8;
            BoundingOutline.Height = ScaledTotalH + 8;

            Contents.Children.Add(BoundingOutline);

            Contents.InvalidateMeasure();

            await Task.CompletedTask;
        }

        private async Task ScreenMonitor()
        {
            Contents.Children.Clear();

            List<(int Left, int Top, int Width, int Height)> Screens = GetScreenBounds();
            int ScreenCount = Screens.Count;

            int SelectedIndex = 0;

            if (SWUS.Screens.Length > 0 && !string.IsNullOrEmpty(SMME.ScreenDevice))
            {
                int DeviceIndex = Array.FindIndex(SWUS.Screens, s => string.Equals(s.szDevice, SMME.ScreenDevice, StringComparison.OrdinalIgnoreCase));

                if (DeviceIndex >= 0)
                {
                    SelectedIndex = DeviceIndex;
                }
            }

            if (string.IsNullOrEmpty(SMME.ScreenDevice) && SWUS.Screens.Length > 0)
            {
                SMMI.EngineSettingManager.SetSetting(SMMCE.ScreenDevice, SWUS.Screens[0].szDevice);
            }

            int MinX = Screens.Min(s => s.Left);
            int MaxX = Screens.Max(s => s.Left + s.Width);
            int MinY = Screens.Min(s => s.Top);
            int MaxY = Screens.Max(s => s.Top + s.Height);

            double TotalW = MaxX - MinX;
            double TotalH = MaxY - MinY;

            double CanvasW = 680;
            double CanvasH = 230;
            double Padding = 15;

            double ScaleX = (CanvasW - (2 * Padding)) / TotalW;
            double ScaleY = (CanvasH - (2 * Padding)) / TotalH;
            double Scale = Math.Min(ScaleX, ScaleY);

            double ScaledTotalW = TotalW * Scale;
            double ScaledTotalH = TotalH * Scale;

            double OffsetX = (CanvasW - ScaledTotalW) / 2.0;
            double OffsetY = (CanvasH - ScaledTotalH) / 2.0;

            for (int Count = 0; Count < ScreenCount; Count++)
            {
                SPVCDS Screen = new();

                if (SelectedIndex == Count)
                {
                    Screen.Border.BorderBrush = Brushes.CornflowerBlue;
                }

                Screen.Index.Text = $"{Count + 1}";

                (int Left, int Top, int Width, int Height) Monitor = Screens[Count];

                double Left = OffsetX + ((Monitor.Left - MinX) * Scale);
                double Top = OffsetY + ((Monitor.Top - MinY) * Scale);
                double Width = Math.Max(25, Monitor.Width * Scale);
                double Height = Math.Max(25, Monitor.Height * Scale);

                Canvas.SetLeft(Screen, Left);
                Canvas.SetTop(Screen, Top);

                Screen.Width = Width;
                Screen.Height = Height;

                Screen.MouseLeftButtonDown += ScreenClicked;

                Contents.Children.Add(Screen);
            }

            Contents.InvalidateMeasure();

            await Task.CompletedTask;
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

        private async Task DuplicateMonitor()
        {
            Contents.Children.Clear();

            List<(int Left, int Top, int Width, int Height)> Screens = GetScreenBounds();

            int MinX = Screens.Min(s => s.Left);
            int MaxX = Screens.Max(s => s.Left + s.Width);
            int MinY = Screens.Min(s => s.Top);
            int MaxY = Screens.Max(s => s.Top + s.Height);

            double TotalW = MaxX - MinX;
            double TotalH = MaxY - MinY;

            double CanvasW = 680;
            double CanvasH = 230;
            double Padding = 15;

            double ScaleX = (CanvasW - (2 * Padding)) / TotalW;
            double ScaleY = (CanvasH - (2 * Padding)) / TotalH;
            double Scale = Math.Min(ScaleX, ScaleY);

            double ScaledTotalW = TotalW * Scale;
            double ScaledTotalH = TotalH * Scale;

            double OffsetX = (CanvasW - ScaledTotalW) / 2.0;
            double OffsetY = (CanvasH - ScaledTotalH) / 2.0;

            for (int Count = 0; Count < Screens.Count; Count++)
            {
                SPVCDD Duplicate = new();

                Duplicate.Index.Text = $"{Count + 1}";

                (int Left, int Top, int Width, int Height) Monitor = Screens[Count];

                double Left = OffsetX + ((Monitor.Left - MinX) * Scale);
                double Top = OffsetY + ((Monitor.Top - MinY) * Scale);
                double Width = Math.Max(25, Monitor.Width * Scale);
                double Height = Math.Max(25, Monitor.Height * Scale);

                Canvas.SetLeft(Duplicate, Left);
                Canvas.SetTop(Duplicate, Top);

                Duplicate.Width = Width;
                Duplicate.Height = Height;

                Contents.Children.Add(Duplicate);
            }

            Contents.InvalidateMeasure();

            await Task.CompletedTask;
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

                        int Index = Convert.ToInt32(Screen.Index.Text) - 1;

                        SWUS.Initialize();

                        if (Index >= 0 && Index < SWUS.Screens.Length)
                        {
                            SMMI.EngineSettingManager.SetSetting(SMMCE.ScreenDevice, SWUS.Screens[Index].szDevice);
                        }
                    }
                    else
                    {
                        Screen.Border.BorderBrush = SRER.GetResource<Brush>("ControlAltFillColorTertiaryBrush");
                    }
                }
            }

            Restart();
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
                GroupName = "DisplayType"
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

        private List<(int Left, int Top, int Width, int Height)> GetScreenBounds()
        {
            SWUS.Initialize();

            return SWUS.Screens.Select(s => (s.rcMonitor.Left, s.rcMonitor.Top, s.rcMonitor.Width, s.rcMonitor.Height)).ToList();
        }

        public void Dispose()
        {
            GC.Collect();
            GC.SuppressFinalize(this);
        }
    }
}