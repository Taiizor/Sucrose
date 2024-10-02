using System.Windows;
using Wpf.Ui.Abstractions.Controls;
using SPMI = Sucrose.Portal.Manage.Internal;
using SPVMPGSVM = Sucrose.Portal.ViewModels.Pages.GeneralSettingViewModel;

namespace Sucrose.Portal.Views.Pages.Setting
{
    /// <summary>
    /// GeneralSettingPage.xaml etkileşim mantığı
    /// </summary>
    public partial class GeneralSettingPage : INavigableView<SPVMPGSVM>, IDisposable
    {
        public SPVMPGSVM ViewModel { get; }

        public GeneralSettingPage(SPVMPGSVM ViewModel)
        {
            this.ViewModel = ViewModel;
            DataContext = this;

            InitializeComponent();

            Culture();
        }

        private void Culture()
        {
            SPMI.CultureService.Dispose();

            SPMI.CultureService = new();

            SPMI.CultureService.CultureCodeChanged += CultureService_CultureCodeChanged;
        }

        private async Task Start()
        {
            foreach (UIElement Content in ViewModel.Contents.ToList())
            {
                FrameSetting.Children.Add(Content);

                await Task.Delay(50);
            }

            await Task.Delay(500);

            FrameSetting.Visibility = Visibility.Visible;
            ProgressSetting.Visibility = Visibility.Collapsed;
        }

        private async void GridSetting_Loaded(object sender, RoutedEventArgs e)
        {
            await Start();
        }

        private async void CultureService_CultureCodeChanged(object sender, EventArgs e)
        {
            FrameSetting.Visibility = Visibility.Collapsed;
            ProgressSetting.Visibility = Visibility.Visible;

            Dispose();

            ViewModel.RefreshInitializeViewModel();

            await Start();
        }

        public void Dispose()
        {
            FrameSetting.Children.Clear();
            ViewModel.Dispose();

            GC.Collect();
            GC.SuppressFinalize(this);
        }
    }
}