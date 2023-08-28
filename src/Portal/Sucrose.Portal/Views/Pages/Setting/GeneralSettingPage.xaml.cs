using System.Windows;
using Wpf.Ui.Controls;
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
        }

        private async Task Start()
        {
            foreach (UIElement Content in ViewModel.Contents)
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

        public void Dispose()
        {
            ViewModel.Dispose();

            GC.Collect();
            GC.SuppressFinalize(this);
        }
    }
}