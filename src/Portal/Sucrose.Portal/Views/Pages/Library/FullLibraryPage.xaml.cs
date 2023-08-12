using Sucrose.Shared.Theme.Helper;
using System.IO;
using System.Windows;
using System.Windows.Controls;
using SMC = Sucrose.Memory.Constant;
using SMMI = Sucrose.Manager.Manage.Internal;
using SPMI = Sucrose.Portal.Manage.Internal;
using SPVCTC = Sucrose.Portal.Views.Controls.ThemeCard;
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
            DataContext = this;

            InitializeComponent();

            SPMI.SearchService.SearchTextChanged += SearchService_SearchTextChanged;
        }

        private async Task AddThemes(string Search)
        {
            ThemeLibrary.Children.Clear();

            if (string.IsNullOrEmpty(Search))
            {
                foreach (string Theme in Themes)
                {
                    SPVCTC ThemeCard = new(Path.GetDirectoryName(Theme), SSTHI.ReadJson(Theme));

                    ThemeCard.IsVisibleChanged += ThemeCard_IsVisibleChanged;

                    ThemeLibrary.Children.Add(ThemeCard);

                    Empty.Visibility = Visibility.Hidden;

                    await Task.Delay(25);
                }
            }
            else
            {
                foreach (string Theme in Themes)
                {
                    SSTHI Info = SSTHI.ReadJson(Theme);
                    string Title = Info.Title.ToLowerInvariant();
                    string Description = Info.Description.ToLowerInvariant();

                    if (Title.Contains(Search) || Description.Contains(Search))
                    {
                        SPVCTC ThemeCard = new(Path.GetDirectoryName(Theme), Info);

                        ThemeCard.IsVisibleChanged += ThemeCard_IsVisibleChanged;

                        ThemeLibrary.Children.Add(ThemeCard);

                        Empty.Visibility = Visibility.Hidden;
                    }
                }
            }

            if (ThemeLibrary.Children.Count <= 0)
            {
                Empty.Visibility = Visibility.Visible;
            }
        }

        private async void FullLibraryPage_Loaded(object sender, RoutedEventArgs e)
        {
            ThemeLibrary.ItemMargin = new Thickness(AdaptiveMargin);
            ThemeLibrary.MaxItemsPerRow = AdaptiveLayout;

            await AddThemes(SPMI.SearchService.SearchText);
        }

        private async void SearchService_SearchTextChanged(object sender, EventArgs e)
        {
            await AddThemes(SPMI.SearchService.SearchText);
        }

        private async void ThemeCard_IsVisibleChanged(object sender, DependencyPropertyChangedEventArgs e)
        {
            await Task.Delay(500);

            if (ThemeLibrary.Children.Count <= 0)
            {
                Empty.Visibility = Visibility.Visible;
            }
        }

        public void Dispose()
        {
            ThemeLibrary.Dispose();

            GC.Collect();
            GC.SuppressFinalize(this);
        }
    }
}