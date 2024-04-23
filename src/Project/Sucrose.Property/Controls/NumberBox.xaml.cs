using System.Windows;
using UserControl = System.Windows.Controls.UserControl;

namespace Sucrose.Property.Controls
{
    /// <summary>
    /// NumberBox.xaml etkileşim mantığı
    /// </summary>
    public partial class NumberBox : UserControl
    {
        public NumberBox()
        {
            DataContext = this;
            InitializeComponent();
        }

        public static readonly DependencyProperty ValueProperty = DependencyProperty.Register("Value", typeof(double), typeof(NumberBox), new PropertyMetadata(1.0));
        public static readonly DependencyProperty HintProperty = DependencyProperty.Register("Hint", typeof(string), typeof(NumberBox), new PropertyMetadata(null));
        public static readonly DependencyProperty HelpProperty = DependencyProperty.Register("Help", typeof(string), typeof(NumberBox), new PropertyMetadata(null));
        public static readonly DependencyProperty MaxProperty = DependencyProperty.Register("Max", typeof(double), typeof(NumberBox), new PropertyMetadata(2.0));
        public static readonly DependencyProperty MinProperty = DependencyProperty.Register("Min", typeof(double), typeof(NumberBox), new PropertyMetadata(0.0));

        public double Value
        {
            get => (double)GetValue(ValueProperty);
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

        public double Max
        {
            get => (double)GetValue(MaxProperty);
            set => SetValue(MaxProperty, value);
        }

        public double Min
        {
            get => (double)GetValue(MinProperty);
            set => SetValue(MinProperty, value);
        }
    }
}