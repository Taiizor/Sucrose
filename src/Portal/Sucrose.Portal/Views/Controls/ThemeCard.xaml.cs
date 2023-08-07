using Sucrose.Shared.Theme.Helper;
using System.IO;
using System.Windows.Controls;
using System.Windows.Media.Imaging;
using SSTHI = Sucrose.Shared.Theme.Helper.Info;
using SHA = Skylark.Helper.Adaptation;
using System.Windows;
using SSSHP = Sucrose.Shared.Space.Helper.Processor;

namespace Sucrose.Portal.Views.Controls
{
    /// <summary>
    /// ThemeCard.xaml etkileşim mantığı
    /// </summary>
    public partial class ThemeCard : UserControl, IDisposable
    {
        private SSTHI Info = null;
        private string Theme = null;

        internal ThemeCard(string Theme, SSTHI Info)
        {
            this.Info = Info;
            this.Theme = Theme;
            InitializeComponent();

            ThemeTitle.Text = SHA.Cut(Info.Title, 30);
            ThemeDescription.Text = SHA.Cut(Info.Description, 30);

            Imagine.ImageSource = new BitmapImage(new Uri(Path.Combine(Theme, Info.Thumbnail)));
        }

        private void MenuFind_Click(object sender, RoutedEventArgs e)
        {
            SSSHP.Run(Theme);
        }

        private void ThemeButton_Click(object sender, RoutedEventArgs e)
        {
            ContextMenu.IsOpen = true;
        }

        public void Dispose()
        {
            GC.SuppressFinalize(this);
        }
    }
}