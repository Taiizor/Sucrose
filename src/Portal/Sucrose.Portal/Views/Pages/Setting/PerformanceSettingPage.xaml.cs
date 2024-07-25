using System.Windows;
using Wpf.Ui.Abstractions.Controls;
using SPVMPPSVM = Sucrose.Portal.ViewModels.Pages.PerformanceSettingViewModel;

namespace Sucrose.Portal.Views.Pages.Setting
{
    /// <summary>
    /// PerformanceSettingPage.xaml etkileşim mantığı
    /// </summary>
    public partial class PerformanceSettingPage : INavigableView<SPVMPPSVM>, IDisposable
    {
        public SPVMPPSVM ViewModel { get; }

        public PerformanceSettingPage(SPVMPPSVM ViewModel)
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
            FrameSetting.Children.Clear();
            ViewModel.Dispose();

            GC.Collect();
            GC.SuppressFinalize(this);
        }
    }
}