using Sucrose.Shared.Theme.Helper;
using System.IO;
using System.Windows.Controls;
using System.Windows.Media.Imaging;
using SSTHI = Sucrose.Shared.Theme.Helper.Info;
using SHA = Skylark.Helper.Adaptation;

namespace Sucrose.Portal.Views.Controls
{
    /// <summary>
    /// ThemeCard.xaml etkileşim mantığı
    /// </summary>
    public partial class ThemeCard : UserControl, IDisposable
    {
        internal ThemeCard(string Theme, SSTHI Info)
        {
            InitializeComponent();

            ThemeTitle.Text = SHA.Cut(Info.Title, 30);
            ThemeDescription.Text = SHA.Cut(Info.Description, 30);

            Imagine.ImageSource = new BitmapImage(new Uri(Path.Combine(Theme, Info.Thumbnail)));
        }

        public void Dispose()
        {
            GC.SuppressFinalize(this);
        }
    }
}