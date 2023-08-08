using Sucrose.Portal.Views.Controls;
using System.IO;
using System.Windows;
using System.Windows.Controls;
using SMC = Sucrose.Memory.Constant;
using SMMI = Sucrose.Manager.Manage.Internal;
using SSTHI = Sucrose.Shared.Theme.Helper.Info;

namespace Sucrose.Portal.Views.Pages.Library
{
    /// <summary>
    /// FullLibraryPage.xaml etkileşim mantığı
    /// </summary>
    public partial class FullLibraryPage : Page, IDisposable
    {
        private static int AdaptiveLayout => SMMI.PortalSettingManager.GetSettingStable(SMC.AdaptiveLayout, 0);

        private static int AdaptiveMargin => SMMI.PortalSettingManager.GetSettingStable(SMC.AdaptiveMargin, 5);

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
                ThemeCard ThemeCard = new(Path.GetDirectoryName(Theme), SSTHI.ReadJson(Theme));

                ThemeLibrary.Children.Add(ThemeCard);

                await Task.Delay(25);
            }

            Themes.Clear();
        }

        private async void FullLibraryPage_Loaded(object sender, RoutedEventArgs e)
        {
            ThemeLibrary.ItemMargin = new Thickness(AdaptiveMargin);
            ThemeLibrary.MaxItemsPerRow = AdaptiveLayout;
            await AddThemes();
        }

        public void Dispose()
        {
            ThemeLibrary.Dispose();

            GC.Collect();
            GC.SuppressFinalize(this);
        }
    }
}