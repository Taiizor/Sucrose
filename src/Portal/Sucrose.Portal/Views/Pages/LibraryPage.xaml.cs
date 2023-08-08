using Sucrose.Portal.ViewModels.Pages;
using Sucrose.Portal.Views.Pages.Library;
using System.IO;
using System.Windows;
using Wpf.Ui.Controls;
using SMC = Sucrose.Memory.Constant;
using SMMI = Sucrose.Manager.Manage.Internal;
using SMR = Sucrose.Memory.Readonly;

namespace Sucrose.Portal.Views.Pages
{
    /// <summary>
    /// LibraryPage.xaml etkileşim mantığı
    /// </summary>
    public partial class LibraryPage : INavigableView<LibraryViewModel>, IDisposable
    {
        private static string Directory => SMMI.EngineSettingManager.GetSetting(SMC.Directory, Path.Combine(SMR.DocumentsPath, SMR.AppName));

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

            if (System.IO.Directory.Exists(Directory))
            {
                string[] Folders = System.IO.Directory.GetDirectories(Directory);

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
                FullLibraryPage Page = new(Themes);

                FrameLibrary.Content = Page;
            }
            else
            {
                EmptyLibraryPage Page = new();

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