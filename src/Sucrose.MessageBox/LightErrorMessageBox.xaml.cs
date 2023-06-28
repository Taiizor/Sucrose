using System.Windows;

namespace Sucrose.MessageBox
{
    /// <summary>
    /// LightErrorMessageBox.xaml etkileşim mantığı
    /// </summary>
    public partial class LightErrorMessageBox : Window
    {
        public string Message { get; set; }

        public LightErrorMessageBox(string Message)
        {
            InitializeComponent();
            ErrorMessageTextBlock.Text = Message;
        }

        private void CloseButton_Click(object sender, RoutedEventArgs e)
        {
            Close();
        }
    }
}