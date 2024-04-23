using System.Windows;
using UserControl = System.Windows.Controls.UserControl;

namespace Sucrose.Property.Controls
{
    /// <summary>
    /// CheckBox.xaml etkileşim mantığı
    /// </summary>
    public partial class CheckBox : UserControl
    {
        public CheckBox()
        {
            DataContext = this;
            InitializeComponent();
        }

        public static readonly DependencyProperty ValueProperty = DependencyProperty.Register("Value", typeof(bool), typeof(CheckBox), new PropertyMetadata(false));
        public static readonly DependencyProperty HelpProperty = DependencyProperty.Register("Help", typeof(string), typeof(CheckBox), new PropertyMetadata(null));
        public static readonly DependencyProperty HintProperty = DependencyProperty.Register("Hint", typeof(string), typeof(CheckBox), new PropertyMetadata(null));

        public bool Value
        {
            get => (bool)GetValue(ValueProperty);
            set => SetValue(ValueProperty, value);
        }

        public string Help
        {
            get => (string)GetValue(HelpProperty);
            set => SetValue(HelpProperty, value);
        }

        public string Hint
        {
            get => (string)GetValue(HintProperty);
            set => SetValue(HintProperty, value);
        }
    }
}