using System.Windows;
using UserControl = System.Windows.Controls.UserControl;

namespace Sucrose.Property.Controls
{
    /// <summary>
    /// Label.xaml etkileşim mantığı
    /// </summary>
    public partial class Label : UserControl
    {
        public Label()
        {
            DataContext = this;
            InitializeComponent();
        }

        public static readonly DependencyProperty ValueProperty = DependencyProperty.Register("Value", typeof(string), typeof(Label), new PropertyMetadata(null));
        public static readonly DependencyProperty HelpProperty = DependencyProperty.Register("Help", typeof(string), typeof(Label), new PropertyMetadata(null));

        public string Value
        {
            get => (string)GetValue(ValueProperty);
            set => SetValue(ValueProperty, value);
        }

        public string Help
        {
            get => (string)GetValue(HelpProperty);
            set => SetValue(HelpProperty, value);
        }
    }
}