using System.Media;
using System.Runtime.InteropServices;
using System.Windows;
using SSDECT = Sucrose.Shared.Dependency.Enum.CommandsType;
using SGHWL = Sucrose.Globalization.Helper.WatchdogLocalization;
using SMR = Sucrose.Memory.Readonly;
using SSHP = Sucrose.Space.Helper.Processor;
using SSMI = Sucrose.Space.Manage.Internal;
using SWHWI = Skylark.Wing.Helper.WindowInterop;
using SWNM = Skylark.Wing.Native.Methods;

namespace Sucrose.Watchdog
{
    /// <summary>
    /// Interaction logic for DarkErrorMessageBox.xaml
    /// </summary>
    public partial class DarkErrorMessageBox : Window
    {
        private static string Path = string.Empty;

        public DarkErrorMessageBox(string ErrorMessage, string LogPath)
        {
            InitializeComponent();

            Path = LogPath;

            SystemSounds.Hand.Play();

            Title = SGHWL.GetValue("WindowTitle");
            Error_Title.Text = SGHWL.GetValue("ErrorTitle");
            Show_Button.Content = SGHWL.GetValue("ShowButton");
            Close_Button.Content = SGHWL.GetValue("CloseButton");
            Error_Message.Text = SGHWL.GetValue("ErrorMessage") + Environment.NewLine + ErrorMessage;

            SourceInitialized += DarkErrorMessageBox_SourceInitialized;
        }

        private void ShowButton_Click(object sender, RoutedEventArgs e)
        {
            SSHP.Run(SSMI.Commandog, $"{SMR.StartCommand}{SSDECT.Log}{SMR.ValueSeparator}{Path}");
        }

        private void CloseButton_Click(object sender, RoutedEventArgs e)
        {
            Close();
        }

        private void DarkErrorMessageBox_SourceInitialized(object sender, EventArgs e)
        {
            bool Value = true;

            SWNM.DwmSetWindowAttribute(SWHWI.Handle(this), SWNM.DWMWindowAttribute.DWMWA_USE_IMMERSIVE_DARK_MODE, ref Value, Marshal.SizeOf(Value));
        }
    }
}