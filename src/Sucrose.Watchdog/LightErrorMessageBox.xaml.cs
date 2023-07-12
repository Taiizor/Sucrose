using System.Media;
using System.Windows;
using SGHWL = Sucrose.Globalization.Helper.WatchdogLocalization;
using SMR = Sucrose.Memory.Readonly;
using SDECT = Sucrose.Dependency.Enum.CommandsType;
using SSHP = Sucrose.Space.Helper.Processor;
using SSMI = Sucrose.Space.Manage.Internal;

namespace Sucrose.Watchdog
{
    /// <summary>
    /// Interaction logic for LightErrorMessageBox.xaml
    /// </summary>
    public partial class LightErrorMessageBox : Window
    {
        private static string Path = string.Empty;

        public LightErrorMessageBox(string ErrorMessage, string LogPath)
        {
            InitializeComponent();

            Path = LogPath;

            SystemSounds.Hand.Play();

            Title = SGHWL.GetValue("WindowTitle");
            Error_Title.Text = SGHWL.GetValue("ErrorTitle");
            Show_Button.Content = SGHWL.GetValue("ShowButton");
            Close_Button.Content = SGHWL.GetValue("CloseButton");
            Error_Message.Text = SGHWL.GetValue("ErrorMessage") + Environment.NewLine + ErrorMessage;
        }

        private void ShowButton_Click(object sender, RoutedEventArgs e)
        {
            SSHP.Run(SSMI.Commandog, $"{SMR.StartCommand}{SDECT.Log}{SMR.ValueSeparator}{Path}");
        }

        private void CloseButton_Click(object sender, RoutedEventArgs e)
        {
            Close();
        }
    }
}