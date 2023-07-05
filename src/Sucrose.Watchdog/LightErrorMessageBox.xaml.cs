using System.Windows;
using SGHWL = Sucrose.Globalization.Helper.WatchdogLocalization;
using SMR = Sucrose.Memory.Readonly;
using SSECT = Sucrose.Space.Enum.CommandsType;
using SSHC = Sucrose.Space.Helper.Command;
using SSMI = Sucrose.Space.Manage.Internal;

namespace Sucrose.Watchdog
{
    /// <summary>
    /// Interaction logic for LightErrorMessageBox.xaml
    /// </summary>
    public partial class LightErrorMessageBox : Window
    {
        private static string Path = string.Empty;

        public LightErrorMessageBox(string Culture, string ErrorMessage, string LogPath)
        {
            InitializeComponent();

            Path = LogPath;

            Title = SGHWL.GetValue("WindowTitle", Culture);
            Error_Title.Text = SGHWL.GetValue("ErrorTitle", Culture);
            Show_Button.Content = SGHWL.GetValue("ShowButton", Culture);
            Close_Button.Content = SGHWL.GetValue("CloseButton", Culture);
            Error_Message.Text = SGHWL.GetValue("ErrorMessage", Culture) + Environment.NewLine + ErrorMessage;
        }

        private void ShowButton_Click(object sender, RoutedEventArgs e)
        {
            SSHC.Run(SSMI.CommandLine, $"{SMR.StartCommand}{SSECT.Log}{SMR.ValueSeparator}{Path}");
        }

        private void CloseButton_Click(object sender, RoutedEventArgs e)
        {
            Close();
        }
    }
}