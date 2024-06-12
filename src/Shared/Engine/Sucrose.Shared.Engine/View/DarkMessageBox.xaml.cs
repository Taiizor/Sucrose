using System.Media;
using System.Windows;
using SSDEDT = Sucrose.Shared.Dependency.Enum.DialogType;
using SSSHD = Sucrose.Shared.Space.Helper.Dark;
using SSSHN = Sucrose.Shared.Space.Helper.Network;
using SSSHS = Sucrose.Shared.Space.Helper.Security;
using SWHWI = Skylark.Wing.Helper.WindowInterop;

namespace Sucrose.Shared.Engine.View
{
    /// <summary>
    /// Interaction logic for DarkMessageBox.xaml
    /// </summary>
    public partial class DarkMessageBox : Window
    {
        internal SSDEDT Result { get; private set; } = SSDEDT.None;

        public DarkMessageBox(string DialogTitle, string DialogMessage, string DialogInfo, string RememberText, string DownloadText, string ContinueText, string CloseText)
        {
            InitializeComponent();

            SystemSounds.Asterisk.Play();

            Title = DialogTitle;
            Dialog_Info.Text = DialogInfo;
            Dialog_Message.Text = DialogMessage;

            Close_Button.Content = CloseText;
            Continue_Button.Content = ContinueText;
            Download_Button.Content = DownloadText;
            Remember_Button.Content = RememberText;

            Download_Button.IsEnabled = SSSHN.GetHostEntry();

            SourceInitialized += DarkMessageBox_SourceInitialized;
        }

        private void CloseButton_Click(object sender, RoutedEventArgs e)
        {
            Result = SSDEDT.Close;

            Close();
        }

        private void ContinueButton_Click(object sender, RoutedEventArgs e)
        {
            Result = SSDEDT.Continue;

            Close();
        }

        private void DownloadButton_Click(object sender, RoutedEventArgs e)
        {
            Result = SSDEDT.Download;

            SSSHS.Apply();

            Close();
        }

        private void RememberButton_Click(object sender, RoutedEventArgs e)
        {
            Result = SSDEDT.Remember;

            Close();
        }

        private void DarkMessageBox_SourceInitialized(object sender, EventArgs e)
        {
            SSSHD.Apply(SWHWI.Handle(this));
        }

        private async void DarkMessageBox_Loaded(object sender, RoutedEventArgs e)
        {
            await Task.Delay(500);

            ShowInTaskbar = true;
        }
    }
}