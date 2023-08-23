using System.IO;
using System.Windows;
using System.Windows.Controls;
using System.Windows.Input;
using SHA = Skylark.Helper.Adaptation;
using SMC = Sucrose.Memory.Constant;
using SMMI = Sucrose.Manager.Manage.Internal;
using SPEIL = Sucrose.Portal.Extension.ImageLoader;
using SPMM = Sucrose.Portal.Manage.Manager;
using SPVCTR = Sucrose.Portal.Views.Controls.ThemeReview;
using SPVCTS = Sucrose.Portal.Views.Controls.ThemeShare;
using SSLHR = Sucrose.Shared.Live.Helper.Run;
using SSSHL = Sucrose.Shared.Space.Helper.Live;
using SSSHP = Sucrose.Shared.Space.Helper.Processor;
using SSTHI = Sucrose.Shared.Theme.Helper.Info;

namespace Sucrose.Portal.Views.Controls
{
    /// <summary>
    /// LibraryCard.xaml etkileşim mantığı
    /// </summary>
    public partial class LibraryCard : UserControl, IDisposable
    {
        private readonly SPEIL Loader = new();
        private readonly string Theme = null;
        private readonly SSTHI Info = null;

        internal LibraryCard(string Theme, SSTHI Info)
        {
            this.Info = Info;
            this.Theme = Theme;

            InitializeComponent();

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

            ThemeTitle.Text = Info.Title.Length > SPMM.TitleLength ? $"{SHA.Cut(Info.Title, SPMM.TitleLength)}..." : Info.Title;
            ThemeDescription.Text = Info.Description.Length > SPMM.DescriptionLength ? $"{SHA.Cut(Info.Description, SPMM.DescriptionLength)}..." : Info.Description;

            string ImagePath = Path.Combine(Theme, Info.Thumbnail);

            if (File.Exists(ImagePath))
            {
                Imagine.ImageSource = Loader.LoadOptimal(ImagePath);
            }

            Dispose();
        }

        private void Use()
        {
            if (SPMM.LibrarySelected != Path.GetFileName(Theme) || !SSSHL.Run())
            {
                SMMI.LibrarySettingManager.SetSetting(SMC.LibrarySelected, Path.GetFileName(Theme));

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

        private void ContextMenu_Opened(object sender, RoutedEventArgs e)
        {
            if (SPMM.LibrarySelected == Path.GetFileName(Theme) && SSSHL.Run())
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

        private void LibraryCard_MouseEnter(object sender, MouseEventArgs e)
        {
            if (SPMM.LibrarySelected == Path.GetFileName(Theme) && SSSHL.Run())
            {
                Cursor = Cursors.Arrow;
            }
            else
            {
                Cursor = Cursors.Hand;
            }
        }

        private void LibraryCard_MouseLeftButtonUp(object sender, MouseButtonEventArgs e)
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