using Sucrose.Shared.Theme.Helper;
using System.IO;
using System.Windows;
using System.Windows.Controls;
using System.Windows.Media.Imaging;
using SHA = Skylark.Helper.Adaptation;
using SMC = Sucrose.Memory.Constant;
using SMMI = Sucrose.Manager.Manage.Internal;
using SSSHP = Sucrose.Shared.Space.Helper.Processor;
using SSTHI = Sucrose.Shared.Theme.Helper.Info;

namespace Sucrose.Portal.Views.Controls
{
    /// <summary>
    /// ThemeCard.xaml etkileşim mantığı
    /// </summary>
    public partial class ThemeCard : UserControl, IDisposable
    {
        private static int DescriptionLength => SMMI.PortalSettingManager.GetSettingStable(SMC.DescriptionLength, 30);

        private static int TitleLength => SMMI.PortalSettingManager.GetSettingStable(SMC.TitleLength, 30);

        private string Theme = null;
        private SSTHI Info = null;

        internal ThemeCard(string Theme, SSTHI Info)
        {
            this.Info = Info;
            this.Theme = Theme;
            InitializeComponent();

            ThemeTitle.Text = SHA.Cut(Info.Title, TitleLength);
            ThemeDescription.Text = SHA.Cut(Info.Description, DescriptionLength);

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