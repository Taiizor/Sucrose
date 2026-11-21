using System.IO;
using System.Windows;
using System.Windows.Controls;
using Wpf.Ui.Controls;
using SELFT = Skylark.Enum.LockScreenFitType;
using SPEIL = Sucrose.Portal.Extension.ImageLoader;
using SPMI = Sucrose.Portal.Manage.Internal;
using SRER = Sucrose.Resources.Extension.Resources;
using SSDEWT = Sucrose.Shared.Dependency.Enum.WallpaperType;
using SSSMI = Sucrose.Shared.Space.Manage.Internal;
using SSTCLC = Sucrose.Shared.Theme.Converter.LocalizationConverter;
using SSTHI = Sucrose.Shared.Theme.Helper.Info;
using SSWEW = Sucrose.Shared.Watchdog.Extension.Watch;
using SWML = Skylark.Wing.Manage.LockScreen;
using SWMLC = Skylark.Wing.Model.LockScreenConfigModel;

namespace Sucrose.Portal.Views.Controls
{
    /// <summary>
    /// ThemeLockScreen.xaml etkileşim mantığı
    /// </summary>
    public partial class ThemeLockScreen : ContentDialog, IDisposable
    {
        private readonly SPEIL Loader = new();
        internal string Theme = string.Empty;
        internal SSTHI Info = new();

        public ThemeLockScreen() : base(SPMI.ContentDialogService.GetDialogHostEx())
        {
            InitializeComponent();
        }

        private async void ContentDialog_Loaded(object sender, RoutedEventArgs e)
        {
            (ThemeTitle.Text, ThemeDescription.Text) = SSTCLC.Convert(Info);

            ThemeTypeIcon.Symbol = Info.Type == SSDEWT.Gif ? SymbolRegular.Image24 : SymbolRegular.Video24;

            string ImagePath = Path.Combine(Theme, Info.Thumbnail);

            if (File.Exists(ImagePath))
            {
                try
                {
                    ThemeThumbnail.Source = Loader.LoadOptimal(ImagePath, true, 300);
                }
                catch (Exception Exception)
                {
                    await SSWEW.Watch_CatchException(Exception);
                }
            }

            FitComboBox.Items.Clear();
            FitComboBox.Items.Add(new ComboBoxItem
            {
                Tag = SELFT.Cover,
                Content = SRER.GetValue("Portal", "ThemeLockScreen", "Fit", "Cover") ?? "Cover"
            });
            FitComboBox.Items.Add(new ComboBoxItem
            {
                Tag = SELFT.Fill,
                Content = SRER.GetValue("Portal", "ThemeLockScreen", "Fit", "Fill") ?? "Fill"
            });
            FitComboBox.Items.Add(new ComboBoxItem
            {
                Tag = SELFT.Fit,
                Content = SRER.GetValue("Portal", "ThemeLockScreen", "Fit", "Fit") ?? "Fit"
            });
            FitComboBox.Items.Add(new ComboBoxItem
            {
                Tag = SELFT.Center,
                Content = SRER.GetValue("Portal", "ThemeLockScreen", "Fit", "Center") ?? "Center"
            });

            SWMLC LockConfig = SWML.LoadConfig();
            string themeMediaPath = Path.Combine(Theme, Info.Source);

            bool isThisThemeActive = LockConfig.Enabled && string.Equals(LockConfig.MediaPath, themeMediaPath, StringComparison.OrdinalIgnoreCase);

            EnableLockScreenSwitch.IsChecked = isThisThemeActive;
            MuteAudioSwitch.IsChecked = LockConfig.Muted;
            MatchDesktopSwitch.IsChecked = LockConfig.MatchDesktopWallpaper;

            FitComboBox.SelectedItem = FitComboBox.Items.Cast<ComboBoxItem>().FirstOrDefault(i => (SELFT)i.Tag == LockConfig.FitMode) ?? FitComboBox.Items[0];

            UpdateCardsState(isThisThemeActive);
        }

        private void EnableLockScreenSwitch_Checked(object sender, RoutedEventArgs e)
        {
            UpdateCardsState(true);
        }

        private void EnableLockScreenSwitch_Unchecked(object sender, RoutedEventArgs e)
        {
            UpdateCardsState(false);
        }

        private void UpdateCardsState(bool isEnabled)
        {
            if (FitRow != null)
            {
                FitRow.IsEnabled = isEnabled;
                FitRow.Opacity = isEnabled ? 1.0 : 0.6;
            }

            if (MuteRow != null)
            {
                MuteRow.IsEnabled = isEnabled;
                MuteRow.Opacity = isEnabled ? 1.0 : 0.6;
            }

            if (MatchDesktopRow != null)
            {
                MatchDesktopRow.IsEnabled = isEnabled;
                MatchDesktopRow.Opacity = isEnabled ? 1.0 : 0.6;
            }
        }

        protected override void OnButtonClick(ContentDialogButton Button)
        {
            if (Button == ContentDialogButton.Primary)
            {
                SWMLC LockConfig = SWML.LoadConfig();
                string themeMediaPath = Path.Combine(Theme, Info.Source);

                if (FitComboBox.SelectedItem is ComboBoxItem selectedItem && selectedItem.Tag is SELFT fit)
                {
                    LockConfig.FitMode = fit;
                }

                LockConfig.Muted = MuteAudioSwitch.IsChecked == true;
                LockConfig.MatchDesktopWallpaper = MatchDesktopSwitch.IsChecked == true;

                if (EnableLockScreenSwitch.IsChecked == true)
                {
                    if (File.Exists(themeMediaPath))
                    {
                        LockConfig.MediaPath = themeMediaPath;
                        LockConfig.Enabled = true;
                        SWML.SaveConfig(LockConfig, true);

                        string corePath = SSSMI.LockScreenCore;
                        if (!SWML.IsProviderRegistered() && File.Exists(corePath))
                        {
                            SWML.RegisterProvider(corePath);
                        }
                    }
                }
                else
                {
                    if (string.Equals(LockConfig.MediaPath, themeMediaPath, StringComparison.OrdinalIgnoreCase))
                    {
                        LockConfig.Enabled = false;
                    }
                    SWML.SaveConfig(LockConfig, false);
                }
            }

            base.OnButtonClick(Button);
        }

        public void Dispose()
        {
            GC.Collect();
            GC.SuppressFinalize(this);
        }
    }
}
