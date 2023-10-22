using System.IO;
using System.Windows;
using System.Windows.Input;
using System.Windows.Media;
using Wpf.Ui.Controls;
using XamlAnimatedGif;
using SPMI = Sucrose.Portal.Manage.Internal;
using SSRER = Sucrose.Shared.Resources.Extension.Resources;

namespace Sucrose.Portal.Views.Controls
{
    /// <summary>
    /// ThemeCreate.xaml etkileşim mantığı
    /// </summary>
    public partial class ThemeCreate : ContentDialog, IDisposable
    {
        public ThemeCreate() : base(SPMI.ContentDialogService.GetContentPresenter())
        {
            InitializeComponent();
        }

        private void Back_Click(object sender, RoutedEventArgs e)
        {
            IsPrimaryButtonEnabled = false;
            GifCard.Visibility = Visibility.Collapsed;
            UrlCard.Visibility = Visibility.Collapsed;
            WebCard.Visibility = Visibility.Collapsed;
            CreateCard.Visibility = Visibility.Visible;
            VideoCard.Visibility = Visibility.Collapsed;
            YouTubeCard.Visibility = Visibility.Collapsed;
            ApplicationCard.Visibility = Visibility.Collapsed;
        }

        private void GifArea_Drop(object sender, DragEventArgs e)
        {
            GifRectangle.Stroke = SSRER.GetResource<Brush>("TextFillColorDisabledBrush");

            if (e.Data.GetDataPresent(DataFormats.FileDrop))
            {
                string[] Files = (string[])e.Data.GetData(DataFormats.FileDrop);

                if (Files.Any())
                {
                    foreach (string Record in Files)
                    {
                        string Extension = Path.GetExtension(Record).ToLowerInvariant();

                        if (Extension == ".gif" || Extension == ".GIF")
                        {
                            AnimationBehavior.SetSourceUri(GifImagine, new(Record));
                            GifDelete.Visibility = Visibility.Visible;
                            GifIcon.Visibility = Visibility.Collapsed;
                            GifText.Visibility = Visibility.Collapsed;
                            GifRectangle.Stroke = Brushes.Transparent;
                            break;
                        }
                    }
                }
            }
        }

        private void GifArea_DragOver(object sender, DragEventArgs e)
        {
            if (!e.Data.GetDataPresent(DataFormats.FileDrop) || e.AllowedEffects.HasFlag(DragDropEffects.Copy) == false)
            {
                e.Effects = DragDropEffects.None;
            }
            else
            {
                e.Effects = DragDropEffects.Copy;
                GifRectangle.Stroke = Brushes.DodgerBlue;
            }
        }

        private void GifArea_DragLeave(object sender, DragEventArgs e)
        {
            if (string.IsNullOrEmpty($"{AnimationBehavior.GetSourceUri(GifImagine)}"))
            {
                GifRectangle.Stroke = SSRER.GetResource<Brush>("TextFillColorDisabledBrush");
            }
            else
            {
                GifRectangle.Stroke = Brushes.Transparent;
            }
        }

        private void GifCreate_Click(object sender, RoutedEventArgs e)
        {
            IsPrimaryButtonEnabled = true;
            GifCard.Visibility = Visibility.Visible;
            CreateCard.Visibility = Visibility.Collapsed;
        }

        private void UrlCreate_Click(object sender, RoutedEventArgs e)
        {
            IsPrimaryButtonEnabled = true;
            UrlCard.Visibility = Visibility.Visible;
            CreateCard.Visibility = Visibility.Collapsed;
        }

        private void WebCreate_Click(object sender, RoutedEventArgs e)
        {
            IsPrimaryButtonEnabled = true;
            WebCard.Visibility = Visibility.Visible;
            CreateCard.Visibility = Visibility.Collapsed;
        }

        private void GifDelete_Click(object sender, RoutedEventArgs e)
        {
            GifIcon.Visibility = Visibility.Visible;
            GifText.Visibility = Visibility.Visible;
            GifDelete.Visibility = Visibility.Collapsed;
            AnimationBehavior.SetSourceUri(GifImagine, null);
            GifRectangle.Stroke = SSRER.GetResource<Brush>("TextFillColorDisabledBrush");
        }

        private void VideoCreate_Click(object sender, RoutedEventArgs e)
        {
            IsPrimaryButtonEnabled = true;
            VideoCard.Visibility = Visibility.Visible;
            CreateCard.Visibility = Visibility.Collapsed;
        }

        private void YouTubeCreate_Click(object sender, RoutedEventArgs e)
        {
            IsPrimaryButtonEnabled = true;
            YouTubeCard.Visibility = Visibility.Visible;
            CreateCard.Visibility = Visibility.Collapsed;
        }

        private void ContentDialog_Loaded(object sender, RoutedEventArgs e)
        {
            //
        }

        private void ApplicationCreate_Click(object sender, RoutedEventArgs e)
        {
            IsPrimaryButtonEnabled = true;
            CreateCard.Visibility = Visibility.Collapsed;
            ApplicationCard.Visibility = Visibility.Visible;
        }

        private void ContentDialog_PreviewKeyDown(object sender, KeyEventArgs e)
        {
            if ((e.Key == Key.Enter || e.Key == Key.Escape) && true)
            {
                e.Handled = true;
            }
        }

        protected override void OnButtonClick(ContentDialogButton Button)
        {
            if (Button == ContentDialogButton.Primary)
            {
                //ilgili tema ekleme kontrolü burada yapılacak
                //ThemeTitle.Focus();
                //return;
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