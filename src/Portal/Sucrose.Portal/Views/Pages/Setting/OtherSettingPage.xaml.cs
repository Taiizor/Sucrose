using System.Windows;
using Wpf.Ui.Controls;
using SPVMPOSVM = Sucrose.Portal.ViewModels.Pages.OtherSettingViewModel;

namespace Sucrose.Portal.Views.Pages.Setting
{
    /// <summary>
    /// OtherSettingPage.xaml etkileşim mantığı
    /// </summary>
    public partial class OtherSettingPage : INavigableView<SPVMPOSVM>, IDisposable
    {
        public SPVMPOSVM ViewModel { get; }

        public OtherSettingPage(SPVMPOSVM ViewModel)
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