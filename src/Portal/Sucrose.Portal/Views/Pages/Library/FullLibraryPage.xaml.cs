using System.IO;
using System.Windows;
using System.Windows.Controls;
using SMML = Sucrose.Manager.Manage.Library;
using SMMP = Sucrose.Manager.Manage.Portal;
using SMMRC = Sucrose.Memory.Manage.Readonly.Content;
using SPMI = Sucrose.Portal.Manage.Internal;
using SPVCLC = Sucrose.Portal.Views.Controls.LibraryCard;
using SSTHI = Sucrose.Shared.Theme.Helper.Info;

namespace Sucrose.Portal.Views.Pages.Library
{
    /// <summary>
    /// FullLibraryPage.xaml etkileşim mantığı
    /// </summary>
    public partial class FullLibraryPage : Page, IDisposable
    {
        private readonly Dictionary<string, string> Searches = new();

        private readonly List<string> Themes = new();

        public FullLibraryPage(Dictionary<string, string> Searches, List<string> Themes)
        {
            this.Themes.AddRange(Themes);
            this.Searches = Searches;
            DataContext = this;

            InitializeComponent();

            Pagination();
        }

        private void Pagination()
        {
            ThemePagination.SelectPageChanged += ThemePagination_SelectPageChanged;
        }

        private async Task AddThemes(string[] Search, int Page)
        {
            Dispose();

            int Count = 0;

            PageScroll.ScrollToVerticalOffset(0);

            ThemePagination.Visibility = Visibility.Collapsed;

            if (Search.Any())
            {
                foreach (KeyValuePair<string, string> Pair in Searches.Where(Theme => Directory.Exists(Path.Combine(SMML.LibraryLocation, Theme.Key))).ToDictionary(Theme => Theme.Key, Theme => Theme.Value).ToArray().Select(Pair => new { Pair.Key, Pair.Value, MatchCount = CountMatchingWords(Pair.Value, Search) }).Where(Pair => Pair.MatchCount > 0).OrderByDescending(Pair => Pair.MatchCount).ToDictionary(Pair => Pair.Key, Pair => Pair.Value))
                {
                    if (SMMP.LibraryPagination * Page > Count && SMMP.LibraryPagination * Page <= Count + SMMP.LibraryPagination)
                    {
                        string ThemePath = Path.Combine(SMML.LibraryLocation, Pair.Key);

                        SSTHI Info = SSTHI.ReadJson(Path.Combine(ThemePath, SMMRC.SucroseInfo));

                        SPVCLC LibraryCard = new(ThemePath, Info);

                        LibraryCard.IsVisibleChanged += (s, e) => ThemeCard_IsVisibleChanged(s, e, Pair.Key);

                        ThemeLibrary.Children.Add(LibraryCard);

                        Empty.Visibility = Visibility.Collapsed;

                        await Task.Delay(50);
                    }

                    Count++;
                }
            }
            else
            {
                foreach (string Theme in Themes.Where(Theme => Directory.Exists(Path.Combine(SMML.LibraryLocation, Theme))).ToList())
                {
                    if (SMMP.LibraryPagination * Page > Count && SMMP.LibraryPagination * Page <= Count + SMMP.LibraryPagination)
                    {
                        string ThemePath = Path.Combine(SMML.LibraryLocation, Theme);

                        SPVCLC LibraryCard = new(ThemePath, SSTHI.ReadJson(Path.Combine(ThemePath, SMMRC.SucroseInfo)));

                        LibraryCard.IsVisibleChanged += (s, e) => ThemeCard_IsVisibleChanged(s, e, Theme);

                        ThemeLibrary.Children.Add(LibraryCard);

                        Empty.Visibility = Visibility.Collapsed;

                        await Task.Delay(50);
                    }

                    Count++;
                }
            }

            if (ThemeLibrary.Children.Count <= 0)
            {
                Empty.Visibility = Visibility.Visible;
            }

            ThemePagination.MaxPage = (int)Math.Ceiling((double)Count / SMMP.LibraryPagination);
        }

        private static int CountMatchingWords(string Text, string[] Pattern)
        {
            return Text.Split(' ').Count(Word => Pattern.Any(Words => Word.Contains(Words)));
        }

        private async void FullLibraryPage_Loaded(object sender, RoutedEventArgs e)
        {
            ThemeLibrary.ItemMargin = new Thickness(SMMP.AdaptiveMargin);
            ThemeLibrary.MaxItemsPerRow = SMMP.AdaptiveLayout;

            await AddThemes(SPMI.SearchService.SearchList, ThemePagination.SelectPage);
        }

        private async void ThemePagination_SelectPageChanged(object sender, EventArgs e)
        {
            Dispose();

            await AddThemes(SPMI.SearchService.SearchList, ThemePagination.SelectPage);
        }

        private async void ThemeCard_IsVisibleChanged(object sender, DependencyPropertyChangedEventArgs e, string Theme)
        {
            if ((bool)e.NewValue == false)
            {
                if ((sender as SPVCLC).Delete)
                {
                    Themes.Remove(Theme);
                    Searches.Remove(Theme);
                }

                await Task.Delay(250);

                if (ThemeLibrary.Children.Count <= 0)
                {
                    if (ThemePagination.MaxPage > ThemePagination.SelectPage)
                    {
                        await AddThemes(SPMI.SearchService.SearchList, ThemePagination.SelectPage);
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