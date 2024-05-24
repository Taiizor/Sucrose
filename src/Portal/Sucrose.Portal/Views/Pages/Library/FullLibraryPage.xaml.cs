using System.IO;
using System.Windows;
using System.Windows.Controls;
using SMMM = Sucrose.Manager.Manage.Manager;
using SMR = Sucrose.Memory.Readonly;
using SPMI = Sucrose.Portal.Manage.Internal;
using SPVCLC = Sucrose.Portal.Views.Controls.LibraryCard;
using SSSHT = Sucrose.Shared.Space.Helper.Tags;
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
            this.Themes.AddRange(Themes);
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

            foreach (string Theme in Themes.ToList())
            {
                string ThemePath = Path.Combine(SMMM.LibraryLocation, Theme);

                if (string.IsNullOrEmpty(Search))
                {
                    if (SMMM.LibraryPagination * Page > Count && SMMM.LibraryPagination * Page <= Count + SMMM.LibraryPagination)
                    {
                        SPVCLC LibraryCard = new(ThemePath, SSTHI.ReadJson(Path.Combine(ThemePath, SMR.SucroseInfo)));

                        LibraryCard.IsVisibleChanged += (s, e) => ThemeCard_IsVisibleChanged(s, e, Theme);

                        ThemeLibrary.Children.Add(LibraryCard);

                        Empty.Visibility = Visibility.Collapsed;

                        await Task.Delay(50);
                    }

                    Count++;
                }
                else
                {
                    SSTHI Info = SSTHI.ReadJson(Path.Combine(ThemePath, SMR.SucroseInfo));

                    string Tags = SSSHT.Join(Info.Tags, SMR.SearchSplit, true, string.Empty);
                    string Description = Info.Description.ToLowerInvariant();
                    string Title = Info.Title.ToLowerInvariant();

                    if (Tags.Contains(Search) || Title.Contains(Search) || Description.Contains(Search))
                    {
                        if (SMMM.LibraryPagination * Page > Count && SMMM.LibraryPagination * Page <= Count + SMMM.LibraryPagination)
                        {
                            SPVCLC LibraryCard = new(ThemePath, Info);

                            LibraryCard.IsVisibleChanged += (s, e) => ThemeCard_IsVisibleChanged(s, e, Theme);

                            ThemeLibrary.Children.Add(LibraryCard);

                            Empty.Visibility = Visibility.Collapsed;

                            await Task.Delay(50);
                        }

                        Count++;
                    }
                }
            }

            if (ThemeLibrary.Children.Count <= 0)
            {
                Empty.Visibility = Visibility.Visible;
            }

            ThemePagination.MaxPage = (int)Math.Ceiling((double)Count / SMMM.LibraryPagination);
        }

        private async void FullLibraryPage_Loaded(object sender, RoutedEventArgs e)
        {
            ThemeLibrary.ItemMargin = new Thickness(SMMM.AdaptiveMargin);
            ThemeLibrary.MaxItemsPerRow = SMMM.AdaptiveLayout;

            await AddThemes(SPMI.SearchService.SearchText, ThemePagination.SelectPage);
        }

        private async void ThemePagination_SelectPageChanged(object sender, EventArgs e)
        {
            Dispose();

            await AddThemes(SPMI.SearchService.SearchText, ThemePagination.SelectPage);
        }

        private async void ThemeCard_IsVisibleChanged(object sender, DependencyPropertyChangedEventArgs e, string Theme)
        {
            if ((bool)e.NewValue == false)
            {
                if ((sender as SPVCLC).Delete)
                {
                    Themes.Remove(Theme);
                    SMMM.Themes.Remove(Theme);
                }

                await Task.Delay(250);

                if (ThemeLibrary.Children.Count <= 0)
                {
                    if (ThemePagination.MaxPage > ThemePagination.SelectPage)
                    {
                        await AddThemes(SPMI.SearchService.SearchText, ThemePagination.SelectPage);
                    }
                    else if (ThemePagination.SelectPage > 0)
                    {
                        ThemePagination.SelectPage--;
                    }
                    else
                    {
                        Empty.Visibility = Visibility.Visible;
                        ThemePagination.Visibility = Visibility.Collapsed;
                    }
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