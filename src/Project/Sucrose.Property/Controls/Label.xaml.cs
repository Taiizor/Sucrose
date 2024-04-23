using System.Windows;
using SSTMLM = Sucrose.Shared.Theme.Model.LabelModel;
using UserControl = System.Windows.Controls.UserControl;

namespace Sucrose.Property.Controls
{
    /// <summary>
    /// Label.xaml etkileşim mantığı
    /// </summary>
    public partial class Label : UserControl
    {
        public static readonly DependencyProperty ValueProperty = DependencyProperty.Register("Value", typeof(string), typeof(Label), new PropertyMetadata(null));

        public static readonly DependencyProperty HelpProperty = DependencyProperty.Register("Help", typeof(string), typeof(Label), new PropertyMetadata(null));
        
        public Label(SSTMLM Data)
        {
            DataContext = this;

            Help = Data.Help;
            Value = Data.Value;

            InitializeComponent();
        }

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