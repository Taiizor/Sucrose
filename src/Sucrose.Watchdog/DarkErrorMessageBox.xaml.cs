using System.Runtime.InteropServices;
using System.Windows;
using SGHWL = Sucrose.Globalization.Helper.WatchdogLocalization;
using SMR = Sucrose.Memory.Readonly;
using SSECT = Sucrose.Space.Enum.CommandsType;
using SSHC = Sucrose.Space.Helper.Command;
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

        public DarkErrorMessageBox(string Culture, string ErrorMessage, string LogPath)
        {
            InitializeComponent();

            Path = LogPath;

            Title = SGHWL.GetValue("WindowTitle", Culture);
            Error_Title.Text = SGHWL.GetValue("ErrorTitle", Culture);
            Show_Button.Content = SGHWL.GetValue("ShowButton", Culture);
            Close_Button.Content = SGHWL.GetValue("CloseButton", Culture);
            Error_Message.Text = SGHWL.GetValue("ErrorMessage", Culture) + Environment.NewLine + ErrorMessage;

            SourceInitialized += DarkErrorMessageBox_SourceInitialized;
        }

        private void ShowButton_Click(object sender, RoutedEventArgs e)
        {
            SSHC.Run(SSMI.CommandLine, $"{SMR.StartCommand}{SSECT.Log}{SMR.ValueSeparator}{Path}");
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