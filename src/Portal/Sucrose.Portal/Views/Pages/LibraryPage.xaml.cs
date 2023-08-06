using Sucrose.Portal.Views.Pages.Library;
using System.IO;
using System.Windows;
using System.Windows.Controls;
using SMC = Sucrose.Memory.Constant;
using SMMI = Sucrose.Manager.Manage.Internal;
using SMR = Sucrose.Memory.Readonly;

namespace Sucrose.Portal.Views.Pages
{
    /// <summary>
    /// LibraryPage.xaml etkileşim mantığı
    /// </summary>
    public partial class LibraryPage : Page
    {
        private static string Directory => SMMI.EngineSettingManager.GetSetting(SMC.Directory, Path.Combine(SMR.DocumentsPath, SMR.AppName));

        public LibraryPage()
        {
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
            LibraryProgress.Visibility = Visibility.Collapsed;
        }

        private void GridLibrary_Loaded(object sender, RoutedEventArgs e)
        {
            _ = Start();
        }
    }
}