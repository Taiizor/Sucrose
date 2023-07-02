using Skylark.Helper;
using Skylark.Wing.Helper;
using Skylark.Wing.Native;
using System.Diagnostics;
using System.Runtime.InteropServices;
using System.Windows;
using SGHMBL = Sucrose.Globalization.Helper.MessageBoxLocalization;

namespace Sucrose.MessageBox
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

            Title = SGHMBL.GetValue("WindowTitle", Culture);
            Error_Title.Text = SGHMBL.GetValue("ErrorTitle", Culture);
            Show_Button.Content = SGHMBL.GetValue("ShowButton", Culture);
            Close_Button.Content = SGHMBL.GetValue("CloseButton", Culture);
            Error_Message.Text = SGHMBL.GetValue("ErrorMessage", Culture) + Environment.NewLine + ErrorMessage;

            SourceInitialized += DarkErrorMessageBox_SourceInitialized;
        }

        private void ShowButton_Click(object sender, RoutedEventArgs e)
        {
            Process.Start(Path);
        }

        private void CloseButton_Click(object sender, RoutedEventArgs e)
        {
            Close();
        }

        private void DarkErrorMessageBox_SourceInitialized(object sender, EventArgs e)
        {
            bool Value = true;

            Methods.DwmSetWindowAttribute(WindowInterop.Handle(this), Methods.DWMWindowAttribute.DWMWA_USE_IMMERSIVE_DARK_MODE, ref Value, Marshal.SizeOf(Value));
        }
    }
}