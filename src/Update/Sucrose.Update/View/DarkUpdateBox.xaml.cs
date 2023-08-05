using System.Media;
using System.Runtime.InteropServices;
using System.Windows;
using SSSHP = Sucrose.Shared.Space.Helper.Processor;
using SWHWI = Skylark.Wing.Helper.WindowInterop;
using SWNM = Skylark.Wing.Native.Methods;

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
                SSSHP.Run(Path);
            }
            else
            {
                Close();
            }
        }

        private void DarkUpdateBox_SourceInitialized(object sender, EventArgs e)
        {
            bool Value = true;

            SWNM.DwmSetWindowAttribute(SWHWI.Handle(this), SWNM.DWMWindowAttribute.DWMWA_USE_IMMERSIVE_DARK_MODE, ref Value, Marshal.SizeOf(Value));
        }
    }
}