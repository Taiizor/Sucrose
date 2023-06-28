using System.Windows;

namespace Sucrose.MessageBox
{
    /// <summary>
    /// DarkErrorMessageBox.xaml etkileşim mantığı
    /// </summary>
    public partial class DarkErrorMessageBox : Window
    {
        public string Message { get; set; }

        public DarkErrorMessageBox(string Message)
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