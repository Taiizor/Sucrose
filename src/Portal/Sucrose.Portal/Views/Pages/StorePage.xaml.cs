using System.IO;
using System.Windows;
using Wpf.Ui.Controls;
using SMC = Sucrose.Memory.Constant;
using SMMI = Sucrose.Manager.Manage.Internal;
using SMR = Sucrose.Memory.Readonly;
using SPMI = Sucrose.Portal.Manage.Internal;
using SPVMPSVM = Sucrose.Portal.ViewModels.Pages.StoreViewModel;
using SPVPSBSP = Sucrose.Portal.Views.Pages.Store.BrokenStorePage;
using SPVPSSSP = Sucrose.Portal.Views.Pages.Store.SearchStorePage;
using SSSHN = Sucrose.Shared.Space.Helper.Network;

namespace Sucrose.Portal.Views.Pages
{
    /// <summary>
    /// StorePage.xaml etkileşim mantığı
    /// </summary>
    public partial class StorePage : INavigableView<SPVMPSVM>, IDisposable
    {
        private static IList<char> Chars => Enumerable.Range('A', 'Z' - 'A' + 1).Concat(Enumerable.Range('a', 'z' - 'a' + 1)).Concat(Enumerable.Range('0', '9' - '0' + 1)).Select(C => (char)C).ToList();

        private static string LibraryLocation => SMMI.EngineSettingManager.GetSetting(SMC.LibraryLocation, Path.Combine(SMR.DocumentsPath, SMR.AppName));

        private static string Agent => SMMI.GeneralSettingManager.GetSetting(SMC.UserAgent, SMR.UserAgent);

        private static string Key => SMMI.PrivateSettingManager.GetSetting(SMC.Key, SMR.Key);

        private static bool Adult => SMMI.PortalSettingManager.GetSetting(SMC.Adult, false);

        private SPVPSBSP BrokenStorePage { get; set; }

        private SPVPSSSP SearchStorePage { get; set; }

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
                SearchStorePage = new();

                FrameStore.Content = SearchStorePage;
            }
            else
            {
                BrokenStorePage = new();

                FrameStore.Content = BrokenStorePage;
            }

            if (!Search)
            {
                await Task.Delay(500);
            }

            FrameStore.Visibility = Visibility.Visible;
            ProgressStore.Visibility = Visibility.Collapsed;
        }

        private async void GridStore_Loaded(object sender, RoutedEventArgs e)
        {
            await Start();
        }

        private async void SearchService_SearchTextChanged(object sender, EventArgs e)
        {
            Dispose();

            FrameStore.Visibility = Visibility.Collapsed;
            ProgressStore.Visibility = Visibility.Visible;

            await Start(true);
        }

        public void Dispose()
        {
            GC.Collect();
            GC.SuppressFinalize(this);
        }
    }
}