using System.Windows;
using Wpf.Ui.Abstractions.Controls;
using SPVMPDSVM = Sucrose.Portal.ViewModels.Pages.DonateSettingViewModel;

namespace Sucrose.Portal.Views.Pages.Setting
{
    /// <summary>
    /// DonateSettingPage.xaml etkileşim mantığı
    /// </summary>
    public partial class DonateSettingPage : INavigableView<SPVMPDSVM>, IDisposable
    {
        public SPVMPDSVM ViewModel { get; }

        public DonateSettingPage(SPVMPDSVM ViewModel)
        {
            this.ViewModel = ViewModel;
            DataContext = this;

            InitializeComponent();
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

        public void Dispose()
        {
            FrameSetting.Children.Clear();
            ViewModel.Dispose();

            GC.Collect();
            GC.SuppressFinalize(this);
        }
    }
}