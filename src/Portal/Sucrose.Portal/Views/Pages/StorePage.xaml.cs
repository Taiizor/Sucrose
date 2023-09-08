using System.IO;
using System.Windows;
using Wpf.Ui.Controls;
using SMR = Sucrose.Memory.Readonly;
using SPMM = Sucrose.Portal.Manage.Manager;
using SPVMPSVM = Sucrose.Portal.ViewModels.Pages.StoreViewModel;
using SPVPSBSP = Sucrose.Portal.Views.Pages.Store.BrokenStorePage;
using SPVPSFSP = Sucrose.Portal.Views.Pages.Store.FullStorePage;
using SPVPSUSP = Sucrose.Portal.Views.Pages.Store.UnknownStorePage;
using SSDESST = Sucrose.Shared.Dependency.Enum.StoreStageType;
using SSSHD = Sucrose.Shared.Store.Helper.Download;
using SSSHN = Sucrose.Shared.Space.Helper.Network;
using SSSHS = Sucrose.Shared.Store.Helper.Store;
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

                if (SSSHD.Store(StoreFile, SPMM.UserAgent, SPMM.Key))
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
            else if (StoreStage == SSDESST.Broken)
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