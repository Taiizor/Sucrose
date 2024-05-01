using System.IO;
using System.Windows;
using Wpf.Ui.Controls;
using SMMM = Sucrose.Manager.Manage.Manager;
using SMR = Sucrose.Memory.Readonly;
using SPVMPSVM = Sucrose.Portal.ViewModels.Pages.StoreViewModel;
using SPVPSBSP = Sucrose.Portal.Views.Pages.Store.BrokenStorePage;
using SPVPSFSP = Sucrose.Portal.Views.Pages.Store.FullStorePage;
using SPVPSUSP = Sucrose.Portal.Views.Pages.Store.UnknownStorePage;
using SSDESST = Sucrose.Shared.Dependency.Enum.StoreStageType;
using SSDEST = Sucrose.Shared.Dependency.Enum.StoreType;
using SSDMM = Sucrose.Shared.Dependency.Manage.Manager;
using SSSHGHD = Sucrose.Shared.Store.Helper.GitHub.Download;
using SSSHN = Sucrose.Shared.Space.Helper.Network;
using SSSHS = Sucrose.Shared.Store.Helper.Store;
using SSSHSD = Sucrose.Shared.Store.Helper.Soferity.Download;
using SSSIR = Sucrose.Shared.Store.Interface.Root;

namespace Sucrose.Portal.Views.Pages
{
    /// <summary>
    /// StorePage.xaml etkileşim mantığı
    /// </summary>
    public partial class StorePage : INavigableView<SPVMPSVM>, IDisposable
    {
        private SPVPSUSP UnknownStorePage { get; set; }

        private SPVPSBSP BrokenStorePage { get; set; }

        private SPVPSFSP FullStorePage { get; set; }

        private SSDESST StoreStage { get; set; }

        public SPVMPSVM ViewModel { get; }

        private SSSIR Root { get; set; }

        public StorePage(SPVMPSVM ViewModel)
        {
            this.ViewModel = ViewModel;
            DataContext = this;

            InitializeComponent();
        }

        private void Stage()
        {
            if (SSSHN.GetHostEntry())
            {
                string StoreFile = Path.Combine(SMR.AppDataPath, SMR.AppName, SMR.CacheFolder, SMR.Store, SMR.StoreFile);

                bool Result = SSDMM.StoreType switch
                {
                    SSDEST.GitHub => SSSHGHD.Store(StoreFile, SMMM.UserAgent, SMMM.Key),
                    _ => SSSHSD.Store(StoreFile, SMMM.UserAgent),
                };

                if (Result)
                {
                    Root = SSSHS.DeserializeRoot(StoreFile);

                    StoreStage = SSDESST.Full;
                }
                else
                {
                    StoreStage = SSDESST.Unknown;
                }
            }
            else
            {
                StoreStage = SSDESST.Broken;
            }
        }

        private async Task Start()
        {
            if (StoreStage == SSDESST.Full)
            {
                FullStorePage = new(Root);

                FrameStore.Content = FullStorePage;
            }
            else if (StoreStage == SSDESST.Unknown)
            {
                UnknownStorePage = new();

                FrameStore.Content = UnknownStorePage;
            }
            else
            {
                BrokenStorePage = new();

                FrameStore.Content = BrokenStorePage;
            }

            await Task.Delay(500);

            FrameStore.Visibility = Visibility.Visible;
            ProgressStore.Visibility = Visibility.Collapsed;
        }

        private async void GridStore_Loaded(object sender, RoutedEventArgs e)
        {
            await Task.Run(Stage);

            await Start();
        }

        public void Dispose()
        {
            FullStorePage?.Dispose();

            GC.Collect();
            GC.SuppressFinalize(this);
        }
    }
}