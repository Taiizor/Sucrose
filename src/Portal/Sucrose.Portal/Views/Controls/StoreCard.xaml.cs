using System.IO;
using System.Windows;
using System.Windows.Controls;
using System.Windows.Input;
using SHA = Skylark.Helper.Adaptation;
using SHS = Skylark.Helper.Skymath;
using SMC = Sucrose.Memory.Constant;
using SMMI = Sucrose.Manager.Manage.Internal;
using SMR = Sucrose.Memory.Readonly;
using SPEIL = Sucrose.Portal.Extension.ImageLoader;
using SPVCTR = Sucrose.Portal.Views.Controls.ThemeReview;
using SPVCTS = Sucrose.Portal.Views.Controls.ThemeShare;
using SSLHR = Sucrose.Shared.Live.Helper.Run;
using SSSHD = Sucrose.Shared.Store.Helper.Download;
using SSSHL = Sucrose.Shared.Space.Helper.Live;
using SSSHP = Sucrose.Shared.Space.Helper.Processor;
using SSSIW = Sucrose.Shared.Store.Interface.Wallpaper;
using SSTHI = Sucrose.Shared.Theme.Helper.Info;

namespace Sucrose.Portal.Views.Controls
{
    /// <summary>
    /// StoreCard.xaml etkileşim mantığı
    /// </summary>
    public partial class StoreCard : UserControl, IDisposable
    {
        private static int DescriptionLength => SHS.Clamp(SMMI.PortalSettingManager.GetSettingStable(SMC.DescriptionLength, 30), 10, int.MaxValue);

        private static int TitleLength => SHS.Clamp(SMMI.PortalSettingManager.GetSettingStable(SMC.TitleLength, 30), 10, int.MaxValue);

        private static string LibrarySelected => SMMI.EngineSettingManager.GetSetting(SMC.LibrarySelected, string.Empty);

        private readonly KeyValuePair<string, SSSIW> Wallpaper = new();
        private readonly SPEIL Loader = new();
        private readonly string Theme = null;
        private readonly string Agent;
        private readonly string Key;
        private SSTHI Info = null;

        internal StoreCard(string Theme, KeyValuePair<string, SSSIW> Wallpaper, string Agent, string Key)
        {
            this.Key = Key;
            this.Agent = Agent;
            this.Theme = Theme;
            this.Wallpaper = Wallpaper;

            InitializeComponent();
        }

        private void Use()
        {
            if (LibrarySelected != Path.GetFileName(Theme) || !SSSHL.Run())
            {
                SMMI.EngineSettingManager.SetSetting(SMC.LibrarySelected, Path.GetFileName(Theme));

                if (SSSHL.Run())
                {
                    SSSHL.Kill();
                }

                SSLHR.Start();

                Cursor = Cursors.Arrow;
            }
        }

        private void MenuUse_Click(object sender, RoutedEventArgs e)
        {
            Use();
        }

        private void MenuFind_Click(object sender, RoutedEventArgs e)
        {
            SSSHP.Run(Theme);
        }

        private async void MenuShare_Click(object sender, RoutedEventArgs e)
        {
            SPVCTS ThemeShare = new()
            {
                Info = Info,
                Theme = Theme
            };
            await ThemeShare.ShowAsync();
            ThemeShare.Dispose();
        }

        private async void MenuReview_Click(object sender, RoutedEventArgs e)
        {
            SPVCTR ThemeReview = new()
            {
                Info = Info,
                Theme = Theme
            };
            await ThemeReview.ShowAsync();
            ThemeReview.Dispose();
        }

        private async void MenuDelete_Click(object sender, RoutedEventArgs e)
        {
            Dispose();
            MinWidth = 0;
            MinHeight = 0;

            Imagine.ImageSource = null;

            Visibility = Visibility.Hidden;

            await Task.Run(() => Directory.Delete(Theme, true));
        }

        private void ThemeMore_Click(object sender, RoutedEventArgs e)
        {
            ContextMenu.IsOpen = true;
        }

        private void DownloadCache()
        {
            if (SSSHD.Cache(Wallpaper, Theme, Agent, Key))
            {
                Info = SSTHI.ReadJson(Path.Combine(Theme, SMR.SucroseInfo));
            }
        }

        private async void StoreCard_Loaded(object sender, RoutedEventArgs e)
        {
            await Task.Run(DownloadCache);

            ToolTip TitleTip = new()
            {
                Content = Info.Title
            };

            ToolTip DescriptionTip = new()
            {
                Content = Info.Description
            };

            ThemeTitle.ToolTip = TitleTip;
            ThemeDescription.ToolTip = DescriptionTip;

            ThemeTitle.Text = Info.Title.Length > TitleLength ? $"{SHA.Cut(Info.Title, TitleLength)}..." : Info.Title;
            ThemeDescription.Text = Info.Description.Length > DescriptionLength ? $"{SHA.Cut(Info.Description, DescriptionLength)}..." : Info.Description;

            string ImagePath = Path.Combine(Theme, Info.Thumbnail);

            if (File.Exists(ImagePath))
            {
                Imagine.ImageSource = Loader.Load(ImagePath);
            }

            await Task.Delay(100);

            Card.Visibility = Visibility.Visible;
            Progress.Visibility = Visibility.Collapsed;
        }

        private void ContextMenu_Opened(object sender, RoutedEventArgs e)
        {
            if (LibrarySelected == Path.GetFileName(Theme) && SSSHL.Run())
            {
                MenuUse.IsEnabled = false;
                MenuDelete.IsEnabled = false;
            }
            else
            {
                MenuUse.IsEnabled = true;
                MenuDelete.IsEnabled = true;
            }
        }

        private void StoreCard_MouseLeftButtonUp(object sender, MouseButtonEventArgs e)
        {
            Use();
        }

        public void Dispose()
        {
            Loader.Dispose();

            GC.Collect();
            GC.SuppressFinalize(this);
        }
    }
}