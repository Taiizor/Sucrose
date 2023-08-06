using Sucrose.Portal.Views.Controls;
using System.IO;
using System.Windows;
using System.Windows.Controls;
using System.Windows.Media;
using System.Windows.Media.Imaging;
using Wpf.Ui.Controls;
using SSTHI = Sucrose.Shared.Theme.Helper.Info;

namespace Sucrose.Portal.Views.Pages.Library
{
    /// <summary>
    /// FullLibraryPage.xaml etkileşim mantığı
    /// </summary>
    public partial class FullLibraryPage : Page, IDisposable
    {
        private List<string> Themes = new();

        public FullLibraryPage(List<string> Themes)
        {
            this.Themes = Themes;
            InitializeComponent();
        }

        private async Task AddThemes()
        {
            foreach (string Theme in Themes)
            {
                SSTHI Info = SSTHI.ReadJson(Theme);

                /*MediaElement gifMediaElement = new()
                {
                    Width = 320,
                    Height = 240,
                    Stretch = Stretch.UniformToFill,
                    Source = new Uri(Path.Combine(Path.GetDirectoryName(Theme), Info.Preview)),
                    LoadedBehavior = MediaState.Manual,
                    UnloadedBehavior = MediaState.Manual,
                    IsMuted = false
                };

                gifMediaElement.MediaEnded += (s, e) =>
                {
                    gifMediaElement.Position = TimeSpan.Zero;
                    gifMediaElement.Play();
                };

                gifMediaElement.Play();

                Card ThemeCard = new()
                {
                    MaxWidth = 360,
                    MinWidth = 260,
                    MinHeight = 160,
                    Background = Brushes.Crimson,
                    VerticalAlignment = VerticalAlignment.Center,
                    HorizontalAlignment = HorizontalAlignment.Center,
                    Content = gifMediaElement
                };

                ThemeLibrary.Children.Add(ThemeCard);*/

                ThemeCard ThemeCard = new(Path.GetDirectoryName(Theme), Info);

                ThemeLibrary.Children.Add(ThemeCard);

                await Task.Delay(25);
            }
        }

        private async void FullLibraryPage_Loaded(object sender, RoutedEventArgs e)
        {
            await AddThemes();
        }

        public void Dispose()
        {
            GC.SuppressFinalize(this);
        }
    }
}