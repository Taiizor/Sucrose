using System.Windows;
using System.Windows.Input;
using Wpf.Ui.Controls;
using SPMI = Sucrose.Portal.Manage.Internal;

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
            //tema oluşturma türü seçildiyse aktif edilecek
            //IsPrimaryButtonEnabled = true;
        }

        private void ApplicationCreate_Click(object sender, RoutedEventArgs e)
        {
            IsPrimaryButtonEnabled = true;
            ApplicationCard.Visibility = Visibility.Visible;
            CreateCard.Visibility = Visibility.Collapsed;
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