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
        }

        private void MenuUse_Click(object sender, RoutedEventArgs e)
        {
            SMMI.EngineSettingManager.SetSetting(SMC.LibrarySelected, Path.GetFileName(Theme));
        }

        private void MenuFind_Click(object sender, RoutedEventArgs e)
        {
            SSSHP.Run(Theme);
        }

        private async void MenuDelete_Click(object sender, RoutedEventArgs e)
        {
            Dispose();
            MinWidth = 0;
            MinHeight = 0;

            Imagine.ImageSource = null;

            Visibility = Visibility.Hidden;

            await Task.Run(() =>
            {
                Directory.Delete(Theme, true);
            });
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