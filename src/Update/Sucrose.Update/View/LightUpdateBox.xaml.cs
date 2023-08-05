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
        private static bool Run = true;
        private static string Path = string.Empty;

        public LightUpdateBox(string BundlePath)
        {
            InitializeComponent();

            Path = BundlePath;

            SystemSounds.Asterisk.Play();
        }

        private void InstallButton_Click(object sender, RoutedEventArgs e)
        {
            if (Run)
            {
                Run = false;
                SSSHP.Run(Path);
            }
            else
            {
                Close();
            }
        }
    }
}