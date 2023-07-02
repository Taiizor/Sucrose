using System.Runtime.InteropServices;
using System.Windows;
using System.Windows.Interop;
using SGHMBL = Sucrose.Globalization.Helper.MessageBoxLocalization;

namespace Sucrose.MessageBox
{
    /// <summary>
    /// Interaction logic for DarkErrorMessageBox.xaml
    /// </summary>
    public partial class DarkErrorMessageBox : Window
    {
        public DarkErrorMessageBox(string Culture, string ErrorMessage)
        {
            InitializeComponent();
            Title = SGHMBL.GetValue("WindowTitle", Culture);
            Error_Title.Text = SGHMBL.GetValue("ErrorTitle", Culture);
            Close_Button.Content = SGHMBL.GetValue("CloseButton", Culture);
            Error_Message.Text = SGHMBL.GetValue("ErrorMessage", Culture) + Environment.NewLine + ErrorMessage;

            SourceInitialized += DarkErrorMessageBox_SourceInitialized;
        }

        private void DarkErrorMessageBox_SourceInitialized(object sender, EventArgs e)
        {
            bool Value = true;

            PInvoke.DwmSetWindowAttribute(new WindowInteropHelper(this).Handle, WMMWindowAttribute.DWMWA_USE_IMMERSIVE_DARK_MODE, ref Value, Marshal.SizeOf(Value));
        }

        private void CloseButton_Click(object sender, RoutedEventArgs e)
        {
            Close();
        }
    }
}