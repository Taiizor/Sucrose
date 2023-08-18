using System.IO;
using System.Windows;
using Wpf.Ui.Controls;
using SMC = Sucrose.Memory.Constant;
using SMMI = Sucrose.Manager.Manage.Internal;
using SMR = Sucrose.Memory.Readonly;
using SPMI = Sucrose.Portal.Manage.Internal;
using SPVMPSVM = Sucrose.Portal.ViewModels.Pages.StoreViewModel;
using SPVPSFSP = Sucrose.Portal.Views.Pages.Store.FullStorePage;
using SPVPSBSP = Sucrose.Portal.Views.Pages.Store.BrokenStorePage;
using SPVPSUSP = Sucrose.Portal.Views.Pages.Store.UnknownStorePage;
using SSSHN = Sucrose.Shared.Space.Helper.Network;
using SSSHD = Sucrose.Shared.Store.Helper.Download;
using SSSHS = Sucrose.Shared.Store.Helper.Store;
using SSSIR = Sucrose.Shared.Store.Interface.Root;

namespace Sucrose.Portal.Views.Pages
{
    /// <summary>
    /// StorePage.xaml etkileşim mantığı
    /// </summary>
    public partial class StorePage : INavigableView<SPVMPSVM>, IDisposable
    {
        private static string Agent => SMMI.GeneralSettingManager.GetSetting(SMC.UserAgent, SMR.UserAgent);

        private static string Key => SMMI.PrivateSettingManager.GetSetting(SMC.Key, SMR.Key);

        private SPVPSUSP UnknownStorePage { get; set; }

        private SPVPSBSP BrokenStorePage { get; set; }

        private SPVPSFSP FullStorePage { get; set; }

        public SPVMPSVM ViewModel { get; }

        public StorePage(SPVMPSVM ViewModel)
        {
            this.ViewModel = ViewModel;
            DataContext = this;

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

        private async Task Start(bool Search = false)
        {
            if (SSSHN.GetHostEntry())
            {
                string StoreFile = Path.Combine(SMR.AppDataPath, SMR.AppName, SMR.CacheFolder, SMR.Store, SMR.StoreFile);

                if (SSSHD.Store(StoreFile, Agent, Key))
                {
                    SSSIR Root = SSSHS.DeserializeRoot(StoreFile);

                    FullStorePage = new(Root);

                    FrameStore.Content = FullStorePage;
                }
                else
                {
                    UnknownStorePage = new();

                    FrameStore.Content = UnknownStorePage;
                }
            }
            else
            {
                BrokenStorePage = new();

                FrameStore.Content = BrokenStorePage;
            }

            if (!Search)
            {
                await Task.Delay(500);

                FrameStore.Visibility = Visibility.Visible;
                ProgressStore.Visibility = Visibility.Collapsed;
            }
        }

        private async void GridStore_Loaded(object sender, RoutedEventArgs e)
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
            GC.Collect();
            GC.SuppressFinalize(this);
        }
    }
}