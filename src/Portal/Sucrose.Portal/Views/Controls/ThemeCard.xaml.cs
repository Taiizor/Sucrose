using Sucrose.Shared.Theme.Helper;
using System.IO;
using System.Windows;
using System.Windows.Controls;
using SHA = Skylark.Helper.Adaptation;
using SHS = Skylark.Helper.Skymath;
using SMC = Sucrose.Memory.Constant;
using SMMI = Sucrose.Manager.Manage.Internal;
using SPEIL = Sucrose.Portal.Extension.ImageLoader;
using SSSHP = Sucrose.Shared.Space.Helper.Processor;
using SSTHI = Sucrose.Shared.Theme.Helper.Info;

namespace Sucrose.Portal.Views.Controls
{
    /// <summary>
    /// ThemeCard.xaml etkileşim mantığı
    /// </summary>
    public partial class ThemeCard : UserControl, IDisposable
    {
        private static int DescriptionLength => SHS.Clamp(SMMI.PortalSettingManager.GetSettingStable(SMC.DescriptionLength, 30), 10, int.MaxValue);

        private static int TitleLength => SHS.Clamp(SMMI.PortalSettingManager.GetSettingStable(SMC.TitleLength, 30), 10, int.MaxValue);

        private string Theme = null;
        private SSTHI Info = null;
        SPEIL Loader = new();

        internal ThemeCard(string Theme, SSTHI Info)
        {
            this.Info = Info;
            this.Theme = Theme;

            InitializeComponent();

            ThemeTitle.Text = Info.Title.Length > TitleLength ? $"{SHA.Cut(Info.Title, TitleLength)}..." : Info.Title;
            ThemeDescription.Text = Info.Description.Length > DescriptionLength ? $"{SHA.Cut(Info.Description, DescriptionLength)}..." : Info.Description;

            Imagine.ImageSource = Loader.Load(Path.Combine(Theme, Info.Thumbnail));
        }

        private void MenuFind_Click(object sender, RoutedEventArgs e)
        {
            SSSHP.Run(Theme);
        }

        private void MenuDelete_Click(object sender, RoutedEventArgs e)
        {
            Dispose();
            MinWidth = 0;
            MinHeight = 0;

            Imagine.ImageSource = null;

            Visibility = Visibility.Hidden;
        }

        private void ThemeButton_Click(object sender, RoutedEventArgs e)
        {
            ContextMenu.IsOpen = true;
        }

        public void Dispose()
        {
            Loader.Dispose();

            GC.Collect();
            GC.SuppressFinalize(this);
        }
    }
}