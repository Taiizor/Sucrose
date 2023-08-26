using System.IO;
using System.Windows;
using Wpf.Ui.Controls;
using SMC = Sucrose.Memory.Constant;
using SMMI = Sucrose.Manager.Manage.Internal;
using SMR = Sucrose.Memory.Readonly;
using SPMI = Sucrose.Portal.Manage.Internal;
using SPMM = Sucrose.Portal.Manage.Manager;
using SPVMPLVM = Sucrose.Portal.ViewModels.Pages.LibraryViewModel;
using SPVPLELP = Sucrose.Portal.Views.Pages.Library.EmptyLibraryPage;
using SPVPLFLP = Sucrose.Portal.Views.Pages.Library.FullLibraryPage;

namespace Sucrose.Portal.Views.Pages
{
    /// <summary>
    /// LibraryPage.xaml etkileşim mantığı
    /// </summary>
    public partial class LibraryPage : INavigableView<SPVMPLVM>, IDisposable
    {
        private List<string> Themes { get; set; } = new();

        private SPVPLELP EmptyLibraryPage { get; set; }

        private SPVPLFLP FullLibraryPage { get; set; }

        public SPVMPLVM ViewModel { get; }

        public LibraryPage(SPVMPLVM ViewModel)
        {
            this.ViewModel = ViewModel;
            DataContext = this;

            InitializeThemes();

            InitializeComponent();

            Search();
        }

        private void Search()
        {
            string Search = SPMI.SearchService.SearchText;

            SPMI.SearchService.Dispose();

            SPMI.SearchService = new()
            {
                SearchText = Search
            };

            SPMI.SearchService.SearchTextChanged += SearchService_SearchTextChanged;
        }

        private void InitializeThemes()
        {
            Themes = SMMI.ThemesManager.GetSetting(SMC.Themes, new List<string>());

            if (Directory.Exists(SPMM.LibraryLocation))
            {
                if (Themes.Any())
                {
                    foreach (string Theme in Themes.ToList())
                    {
                        string ThemePath = Path.Combine(SPMM.LibraryLocation, Theme);
                        string InfoPath = Path.Combine(ThemePath, SMR.SucroseInfo);

                        if (!Directory.Exists(ThemePath) || !File.Exists(InfoPath))
                        {
                            Themes.Remove(Theme);

                            if (Directory.Exists(ThemePath))
                            {
                                Directory.Delete(ThemePath, true);
                            }
                        }
                    }
                }

                string[] Folders = Directory.GetDirectories(SPMM.LibraryLocation);

                if (Folders.Any())
                {
                    foreach (string Folder in Folders)
                    {
                        string InfoPath = Path.Combine(Folder, SMR.SucroseInfo);

                        if (File.Exists(InfoPath))
                        {
                            if (!Themes.Contains(Path.GetFileName(Folder)))
                            {
                                Themes.Add(Path.GetFileName(Folder));
                            }
                        }
                        else
                        {
                            Directory.Delete(Folder, true);
                        }
                    }
                }
            }
            else
            {
                Themes.Clear();
            }

            SMMI.ThemesManager.SetSetting(SMC.Themes, Themes);
        }

        private async Task Start(bool Progress = false)
        {
            if (Themes.Any())
            {
                FullLibraryPage = new(Themes);

                FrameLibrary.Content = FullLibraryPage;
            }
            else
            {
                EmptyLibraryPage = new();

                FrameLibrary.Content = EmptyLibraryPage;
            }

            if (!Progress)
            {
                await Task.Delay(500);

                FrameLibrary.Visibility = Visibility.Visible;
                ProgressLibrary.Visibility = Visibility.Collapsed;
            }
        }

        private async void GridLibrary_Loaded(object sender, RoutedEventArgs e)
        {
            await Start();
        }

        private async void SearchService_SearchTextChanged(object sender, EventArgs e)
        {
            Dispose();

            await Start(true);
        }

        public void Dispose()
        {
            FullLibraryPage?.Dispose();
            EmptyLibraryPage?.Dispose();

            GC.Collect();
            GC.SuppressFinalize(this);
        }
    }
}