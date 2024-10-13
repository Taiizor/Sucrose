using System.Media;
using System.Windows;
using SMMRG = Sucrose.Memory.Manage.Readonly.General;
using SMMRU = Sucrose.Memory.Manage.Readonly.Url;
using SSDECT = Sucrose.Shared.Dependency.Enum.CommandType;
using SSSHD = Sucrose.Shared.Space.Helper.Dark;
using SSSHP = Sucrose.Shared.Space.Helper.Processor;
using SSSMI = Sucrose.Shared.Space.Manage.Internal;
using SWHWI = Skylark.Wing.Helper.WindowInterop;

namespace Sucrose.Watchdog.View
{
    /// <summary>
    /// Interaction logic for DarkErrorMessageBox.xaml
    /// </summary>
    public partial class DarkErrorMessageBox : Window
    {
        private static string Path = string.Empty;
        private static string Text = string.Empty;
        private static string Source = string.Empty;

        public DarkErrorMessageBox(string ErrorMessage, string LogPath, string HelpSource = null, string HelpText = null)
        {
            InitializeComponent();

            Path = LogPath;
            Text = HelpText;
            Source = HelpSource;

            SystemSounds.Hand.Play();

            if (!string.IsNullOrEmpty(Source))
            {
                Help_Button.Content = Text;
            }

            Error_Message.Text += Environment.NewLine + ErrorMessage;

            SourceInitialized += DarkErrorMessageBox_SourceInitialized;
        }

        private void ShowButton_Click(object sender, RoutedEventArgs e)
        {
            SSSHP.Run(SSSMI.Commandog, $"{SMMRG.StartCommand}{SSDECT.Log}{SMMRG.ValueSeparator}{Path}");
        }

        private void HelpButton_Click(object sender, RoutedEventArgs e)
        {
            if (string.IsNullOrEmpty(Source))
            {
                SSSHP.Run(SSSMI.Commandog, $"{SMMRG.StartCommand}{SSDECT.Wiki}{SMMRG.ValueSeparator}{SMMRU.GitHubSucroseWiki}");
            }
            else
            {
                SSSHP.Run(SSSMI.Commandog, $"{SMMRG.StartCommand}{SSDECT.Wiki}{SMMRG.ValueSeparator}{Source}");
            }
        }

        private void CloseButton_Click(object sender, RoutedEventArgs e)
        {
            Close();
        }

        private void DarkErrorMessageBox_SourceInitialized(object sender, EventArgs e)
        {
            SSSHD.Apply(SWHWI.Handle(this));
        }

        private async void DarkErrorMessageBox_Loaded(object sender, RoutedEventArgs e)
        {
            await Task.Delay(500);

            ShowInTaskbar = true;
        }
    }
}