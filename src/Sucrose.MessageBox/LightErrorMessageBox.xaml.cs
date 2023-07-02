using System.Diagnostics;
using System.Windows;
using SGHMBL = Sucrose.Globalization.Helper.MessageBoxLocalization;

namespace Sucrose.MessageBox
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

            Title = SGHMBL.GetValue("WindowTitle", Culture);
            Error_Title.Text = SGHMBL.GetValue("ErrorTitle", Culture);
            Show_Button.Content = SGHMBL.GetValue("ShowButton", Culture);
            Close_Button.Content = SGHMBL.GetValue("CloseButton", Culture);
            Error_Message.Text = SGHMBL.GetValue("ErrorMessage", Culture) + Environment.NewLine + ErrorMessage;
        }

        private void ShowButton_Click(object sender, RoutedEventArgs e)
        {
            Process.Start(Path);
        }

        private void CloseButton_Click(object sender, RoutedEventArgs e)
        {
            Close();
        }
    }
}