using System.IO;
using System.Windows;
using Wpf.Ui.Abstractions.Controls;
using SMMG = Sucrose.Manager.Manage.General;
using SMMO = Sucrose.Manager.Manage.Objectionable;
using SMMRC = Sucrose.Memory.Manage.Readonly.Content;
using SMMRF = Sucrose.Memory.Manage.Readonly.Folder;
using SMMRG = Sucrose.Memory.Manage.Readonly.General;
using SMMRP = Sucrose.Memory.Manage.Readonly.Path;
using SPVMPSVM = Sucrose.Portal.ViewModels.Pages.StoreViewModel;
using SPVPSBSP = Sucrose.Portal.Views.Pages.Store.BrokenStorePage;
using SPVPSFSP = Sucrose.Portal.Views.Pages.Store.FullStorePage;
using SPVPSUSP = Sucrose.Portal.Views.Pages.Store.UnknownStorePage;
using SSDESSET = Sucrose.Shared.Dependency.Enum.StoreStageType;
using SSDESSRT = Sucrose.Shared.Dependency.Enum.StoreServerType;
using SSDMMP = Sucrose.Shared.Dependency.Manage.Manager.Portal;
using SSSHGHD = Sucrose.Shared.Store.Helper.GitHub.Download;
using SSSHN = Sucrose.Shared.Space.Helper.Network;
using SSSHS = Sucrose.Shared.Store.Helper.Store;
using SSSHSD = Sucrose.Shared.Store.Helper.Soferity.Download;
using SSSIS = Sucrose.Shared.Store.Interface.Store;

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

        private SSDESSET StoreStage { get; set; }

        public SPVMPSVM ViewModel { get; }

        private SSSIS Store { get; set; }

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
                string PatternFile = Path.Combine(SMMRP.ApplicationData, SMMRG.AppName, SMMRF.Cache, SMMRF.Store, SMMRC.PatternFile);
                string StoreFile = Path.Combine(SMMRP.ApplicationData, SMMRG.AppName, SMMRF.Cache, SMMRF.Store, SMMRC.StoreFile);

                bool Result = SSDMMP.StoreServerType switch
                {
                    SSDESSRT.GitHub => SSSHGHD.Store(StoreFile, SMMG.UserAgent, SMMO.Key),
                    _ => SSSHSD.Store(StoreFile, SMMG.UserAgent)
                };

                if (Result)
                {
                    if (SSDMMP.StoreServerType == SSDESSRT.Soferity && SSSHSD.Pattern(PatternFile, SMMG.UserAgent))
                    {
                        Store = SSSHS.ReadJson(PatternFile);
                    }
                    else
                    {
                        Store = SSSHS.ReadJson(StoreFile);
                    }

                    StoreStage = SSDESSET.Full;
                }
                else
                {
                    StoreStage = SSDESSET.Unknown;
                }
            }
            else
            {
                StoreStage = SSDESSET.Broken;
            }
        }

        private async Task Start()
        {
            if (StoreStage == SSDESSET.Full)
            {
                FullStorePage = new(Store);

                FrameStore.Content = FullStorePage;
            }
            else if (StoreStage == SSDESSET.Unknown)
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