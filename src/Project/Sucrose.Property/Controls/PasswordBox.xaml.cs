using System.Windows;
using UserControl = System.Windows.Controls.UserControl;

namespace Sucrose.Property.Controls
{
    /// <summary>
    /// PasswordBox.xaml etkileşim mantığı
    /// </summary>
    public partial class PasswordBox : UserControl
    {
        public PasswordBox()
        {
            DataContext = this;
            InitializeComponent();
        }

        public static readonly DependencyProperty ValueProperty = DependencyProperty.Register("Value", typeof(string), typeof(PasswordBox), new PropertyMetadata(null));
        public static readonly DependencyProperty HintProperty = DependencyProperty.Register("Hint", typeof(string), typeof(PasswordBox), new PropertyMetadata(null));
        public static readonly DependencyProperty HelpProperty = DependencyProperty.Register("Help", typeof(string), typeof(PasswordBox), new PropertyMetadata(null));

        public string Value
        {
            get => (string)GetValue(ValueProperty);
            set => SetValue(ValueProperty, value);
        }

        public string Hint
        {
            get => (string)GetValue(HintProperty);
            set => SetValue(HintProperty, value);
        }

        public string Help
        {
            get => (string)GetValue(HelpProperty);
            set => SetValue(HelpProperty, value);
        }
    }
}