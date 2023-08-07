using System.Windows;
using System.Windows.Controls;

namespace Sucrose.Portal.Views.Pages
{
    /// <summary>
    /// SettingPage.xaml etkileşim mantığı
    /// </summary>
    public partial class SettingPage : Page
    {
        public SettingPage()
        {
            InitializeComponent();
        }

        private async Task Start()
        {
            var Page = new Page();

            FrameSetting.Content = Page;

            await Task.Delay(500000);

            FrameSetting.Visibility = Visibility.Visible;
            ProgressSetting.Visibility = Visibility.Collapsed;
        }

        private async void GridSetting_Loaded(object sender, RoutedEventArgs e)
        {
            await Start();
        }
    }
}
