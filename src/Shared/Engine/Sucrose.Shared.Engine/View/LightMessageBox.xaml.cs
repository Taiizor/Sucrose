using System.Media;
using System.Windows;
using SSDEDT = Sucrose.Shared.Dependency.Enum.DialogType;
using SSSHN = Sucrose.Shared.Space.Helper.Network;
using SSSHS = Sucrose.Shared.Space.Helper.Security;

namespace Sucrose.Shared.Engine.View
{
    /// <summary>
    /// Interaction logic for LightMessageBox.xaml
    /// </summary>
    public partial class LightMessageBox : Window
    {
        internal SSDEDT Result { get; private set; } = SSDEDT.None;

        public LightMessageBox(string DialogTitle, string DialogMessage, string DialogInfo, string DownloadText, string ContinueText, string CloseText)
        {
            InitializeComponent();

            SystemSounds.Asterisk.Play();

            Title = DialogTitle;
            Dialog_Info.Text = DialogInfo;
            Dialog_Message.Text = DialogMessage;

            Close_Button.Content = CloseText;
            Continue_Button.Content = ContinueText;
            Download_Button.Content = DownloadText;

            Download_Button.IsEnabled = SSSHN.GetHostEntry();
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

        private async void LightMessageBox_Loaded(object sender, RoutedEventArgs e)
        {
            await Task.Delay(500);

            ShowInTaskbar = true;
        }
    }
}