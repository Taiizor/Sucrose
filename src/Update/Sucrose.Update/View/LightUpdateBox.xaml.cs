using System.Media;
using System.Windows;
using SSSHP = Sucrose.Shared.Space.Helper.Processor;

namespace Sucrose.Update.View
{
    /// <summary>
    /// Interaction logic for LightUpdateBox.xaml
    /// </summary>
    public partial class LightUpdateBox : Window
    {
        private static string Path = string.Empty;

        public LightUpdateBox(string BundlePath)
        {
            InitializeComponent();

            Path = BundlePath;

            SystemSounds.Asterisk.Play();
        }

        private async void InstallButton_Click(object sender, RoutedEventArgs e)
        {
            Install_Button.IsEnabled = false;

            await Task.Run(() => SSSHP.Run(Path));

            await Task.Delay(TimeSpan.FromSeconds(10));

            Install_Button.IsEnabled = true;
        }
    }
}