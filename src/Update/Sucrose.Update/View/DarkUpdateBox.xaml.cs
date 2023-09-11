using System.Media;
using System.Windows;
using SMR = Sucrose.Memory.Readonly;
using SSDECT = Sucrose.Shared.Dependency.Enum.CommandsType;
using SSSHD = Sucrose.Shared.Space.Helper.Dark;
using SSSHP = Sucrose.Shared.Space.Helper.Processor;
using SSSMI = Sucrose.Shared.Space.Manage.Internal;
using SWHWI = Skylark.Wing.Helper.WindowInterop;

namespace Sucrose.Update.View
{
    /// <summary>
    /// Interaction logic for DarkUpdateBox.xaml
    /// </summary>
    public partial class DarkUpdateBox : Window
    {
        private static bool Run = true;
        private static string Path = string.Empty;

        public DarkUpdateBox(string BundlePath)
        {
            InitializeComponent();

            Path = BundlePath;

            SystemSounds.Asterisk.Play();

            SourceInitialized += DarkUpdateBox_SourceInitialized;
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

        private void DarkUpdateBox_SourceInitialized(object sender, EventArgs e)
        {
            SSSHD.Apply(SWHWI.Handle(this));
        }
    }
}