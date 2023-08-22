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

            Pagination();
        }

        private void Pagination()
        {
            ThemePagination.SelectPageChanged += ThemePagination_SelectPageChanged;
        }

        private async Task AddThemes(string Search, int Page)
        {
            Dispose();

            int Count = 0;

            PageScroll.ScrollToVerticalOffset(0);

            ThemePagination.Visibility = Visibility.Collapsed;

            foreach (string Theme in Themes)
            {
                if (string.IsNullOrEmpty(Search))
                {
                    if (SPMM.LibraryPagination * Page > Count && SPMM.LibraryPagination * Page <= Count + SPMM.LibraryPagination)
                    {
                        SPVCLC LibraryCard = new(Path.GetDirectoryName(Theme), SSTHI.ReadJson(Theme));

                        LibraryCard.IsVisibleChanged += ThemeCard_IsVisibleChanged;

                        ThemeLibrary.Children.Add(LibraryCard);

                        Empty.Visibility = Visibility.Collapsed;

                        await Task.Delay(25);
                    }

                    Count++;
                }
                else
                {
                    SSTHI Info = SSTHI.ReadJson(Theme);
                    string Title = Info.Title.ToLowerInvariant();
                    string Description = Info.Description.ToLowerInvariant();

                    if (Title.Contains(Search) || Description.Contains(Search))
                    {
                        if (SPMM.LibraryPagination * Page > Count && SPMM.LibraryPagination * Page <= Count + SPMM.LibraryPagination)
                        {
                            SPVCLC LibraryCard = new(Path.GetDirectoryName(Theme), Info);

                            LibraryCard.IsVisibleChanged += ThemeCard_IsVisibleChanged;

                            ThemeLibrary.Children.Add(LibraryCard);

                            Empty.Visibility = Visibility.Collapsed;

                            await Task.Delay(25);
                        }

                        Count++;
                    }
                }
            }

            if (ThemeLibrary.Children.Count <= 0)
            {
                Empty.Visibility = Visibility.Visible;
            }

            ThemePagination.MaxPage = (int)Math.Ceiling((double)Count / SPMM.LibraryPagination);
        }

        private async void FullLibraryPage_Loaded(object sender, RoutedEventArgs e)
        {
            ThemeLibrary.ItemMargin = new Thickness(SPMM.AdaptiveMargin);
            ThemeLibrary.MaxItemsPerRow = SPMM.AdaptiveLayout;

            await AddThemes(SPMI.SearchService.SearchText, ThemePagination.SelectPage);
        }

        private async void ThemePagination_SelectPageChanged(object sender, EventArgs e)
        {
            Dispose();

            await AddThemes(SPMI.SearchService.SearchText, ThemePagination.SelectPage);
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