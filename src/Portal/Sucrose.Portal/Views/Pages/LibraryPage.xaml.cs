using Sucrose.Portal.ViewModels.Pages;
using System.IO;
using System.Windows;
using Wpf.Ui.Controls;
using SMC = Sucrose.Memory.Constant;
using SMMI = Sucrose.Manager.Manage.Internal;
using SMR = Sucrose.Memory.Readonly;
using SPMI = Sucrose.Portal.Manage.Internal;
using SPVPLELP = Sucrose.Portal.Views.Pages.Library.EmptyLibraryPage;
using SPVPLFLP = Sucrose.Portal.Views.Pages.Library.FullLibraryPage;

namespace Sucrose.Portal.Views.Pages
{
    /// <summary>
    /// LibraryPage.xaml etkileşim mantığı
    /// </summary>
    public partial class LibraryPage : INavigableView<LibraryViewModel>, IDisposable
    {
        private static string LibraryLocation => SMMI.EngineSettingManager.GetSetting(SMC.LibraryLocation, Path.Combine(SMR.DocumentsPath, SMR.AppName));

        private SPVPLELP EmptyLibraryPage { get; set; }

        private SPVPLFLP FullLibraryPage { get; set; }

        public LibraryViewModel ViewModel { get; }

        public LibraryPage(LibraryViewModel ViewModel)
        {
            this.ViewModel = ViewModel;
            DataContext = this;
            InitializeComponent();

            SPMI.SearchService.SearchTextChanged += SearchService_SearchTextChanged;
        }

        private async void SearchService_SearchTextChanged(object sender, EventArgs e)
        {
            Dispose();

            FrameLibrary.Visibility = Visibility.Collapsed;
            ProgressLibrary.Visibility = Visibility.Visible;

            await Start(true);
        }

        private async Task Start(bool Search = false)
        {
            List<string> Themes = new();

            if (Directory.Exists(LibraryLocation))
            {
                string[] Folders = Directory.GetDirectories(LibraryLocation);

                if (Folders.Any())
                {
                    foreach (string Folder in Folders)
                    {
                        string InfoPath = Path.Combine(Folder, SMR.SucroseInfo);

                        if (File.Exists(InfoPath))
                        {
                            Themes.Add(InfoPath);
                        }
                    }
                }
            }

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

            if (!Search)
            {
                await Task.Delay(500);
            }

            FrameLibrary.Visibility = Visibility.Visible;
            ProgressLibrary.Visibility = Visibility.Collapsed;
        }

        private async void GridLibrary_Loaded(object sender, RoutedEventArgs e)
        {
            await Start();
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