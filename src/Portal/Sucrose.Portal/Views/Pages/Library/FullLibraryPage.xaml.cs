using System.IO;
using System.Windows;
using System.Windows.Controls;
using SPMI = Sucrose.Portal.Manage.Internal;
using SPMM = Sucrose.Portal.Manage.Manager;
using SPVCLC = Sucrose.Portal.Views.Controls.LibraryCard;
using SSTHI = Sucrose.Shared.Theme.Helper.Info;

namespace Sucrose.Portal.Views.Pages.Library
{
    /// <summary>
    /// FullLibraryPage.xaml etkileşim mantığı
    /// </summary>
    public partial class FullLibraryPage : Page, IDisposable
    {
        private readonly List<string> Themes = new();

        public FullLibraryPage(List<string> Themes)
        {
            this.Themes = Themes;
            DataContext = this;

            InitializeComponent();
        }

        private async Task AddThemes(string Search)
        {
            Dispose();

            foreach (string Theme in Themes)
            {
                if (string.IsNullOrEmpty(Search))
                {
                    SPVCLC LibraryCard = new(Path.GetDirectoryName(Theme), SSTHI.ReadJson(Theme));

                    LibraryCard.IsVisibleChanged += ThemeCard_IsVisibleChanged;

                    ThemeLibrary.Children.Add(LibraryCard);

                    Empty.Visibility = Visibility.Collapsed;

                    await Task.Delay(25);
                }
                else
                {
                    SSTHI Info = SSTHI.ReadJson(Theme);
                    string Title = Info.Title.ToLowerInvariant();
                    string Description = Info.Description.ToLowerInvariant();

                    if (Title.Contains(Search) || Description.Contains(Search))
                    {
                        SPVCLC LibraryCard = new(Path.GetDirectoryName(Theme), Info);

                        LibraryCard.IsVisibleChanged += ThemeCard_IsVisibleChanged;

                        ThemeLibrary.Children.Add(LibraryCard);

                        Empty.Visibility = Visibility.Collapsed;

                        await Task.Delay(25);
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
            ThemeLibrary.ItemMargin = new Thickness(SPMM.AdaptiveMargin);
            ThemeLibrary.MaxItemsPerRow = SPMM.AdaptiveLayout;

            await AddThemes(SPMI.SearchService.SearchText);
        }

        private async void ThemeCard_IsVisibleChanged(object sender, DependencyPropertyChangedEventArgs e)
        {
            if ((bool)e.NewValue == false)
            {
                await Task.Delay(250);

                if (ThemeLibrary.Children.Count <= 0)
                {
                    Empty.Visibility = Visibility.Visible;
                }
            }
        }

        public void Dispose()
        {
            ThemeLibrary.Children.Clear();

            GC.Collect();
            GC.SuppressFinalize(this);
        }
    }
}