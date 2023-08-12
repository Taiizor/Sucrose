using Sucrose.Portal.ViewModels.Pages;
using System.IO;
using System.Windows;
using Wpf.Ui.Controls;
using SMC = Sucrose.Memory.Constant;
using SMMI = Sucrose.Manager.Manage.Internal;
using SMR = Sucrose.Memory.Readonly;
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

        public LibraryViewModel ViewModel { get; }

        public LibraryPage(LibraryViewModel ViewModel)
        {
            this.ViewModel = ViewModel;
            DataContext = this;

            InitializeComponent();
        }

        private async Task Start()
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
                SPVPLFLP Page = new(Themes);

                FrameLibrary.Content = Page;
            }
            else
            {
                SPVPLELP Page = new();

                FrameLibrary.Content = Page;
            }

            await Task.Delay(500);

            FrameLibrary.Visibility = Visibility.Visible;
            ProgressLibrary.Visibility = Visibility.Collapsed;
        }

        private async void GridLibrary_Loaded(object sender, RoutedEventArgs e)
        {
            await Start();
        }

        public void Dispose()
        {
            GC.Collect();
            GC.SuppressFinalize(this);
        }
    }
}