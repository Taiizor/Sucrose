using System.Media;
using System.Windows;
using SMR = Sucrose.Memory.Readonly;
using SSDECT = Sucrose.Shared.Dependency.Enum.CommandsType;
using SSSHP = Sucrose.Shared.Space.Helper.Processor;
using SSSMI = Sucrose.Shared.Space.Manage.Internal;

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
                SSSHP.Run(SSSMI.Commandog, $"{SMR.StartCommand}{SSDECT.Bundle}{SMR.ValueSeparator}{Path}");
            }
            else
            {
                Close();
            }
        }
    }
}