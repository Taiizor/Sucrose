using System.Collections.ObjectModel;
using System.Windows;
using UserControl = System.Windows.Controls.UserControl;

namespace Sucrose.Property.Controls
{
    /// <summary>
    /// DropDown.xaml etkileşim mantığı
    /// </summary>
    public partial class DropDown : UserControl
    {
        public ObservableCollection<string> ItemsSource { get; set; } = new ObservableCollection<string>();

        public DropDown()
        {
            InitializeComponent();
            DataContext = this;
        }

        public static readonly DependencyProperty ItemsProperty = DependencyProperty.Register("Items", typeof(string), typeof(DropDown), new PropertyMetadata(null, OnItemsPropertyChanged));
        public static readonly DependencyProperty TextProperty = DependencyProperty.Register("Text", typeof(string), typeof(DropDown), new PropertyMetadata(null));
        public static readonly DependencyProperty HelpProperty = DependencyProperty.Register("Help", typeof(string), typeof(DropDown), new PropertyMetadata(null));
        public static readonly DependencyProperty ValueProperty = DependencyProperty.Register("Value", typeof(int), typeof(DropDown), new PropertyMetadata(0));

        public string Items
        {
            get => (string)GetValue(ItemsProperty);
            set => SetValue(ItemsProperty, value);
        }

        public string Text
        {
            get => (string)GetValue(TextProperty);
            set => SetValue(TextProperty, value);
        }

        public string Help
        {
            get => (string)GetValue(HelpProperty);
            set => SetValue(HelpProperty, value);
        }

        public int Value
        {
            get => (int)GetValue(ValueProperty);
            set => SetValue(ValueProperty, value);
        }

        private static void OnItemsPropertyChanged(DependencyObject d, DependencyPropertyChangedEventArgs e)
        {
            if (d is DropDown control)
            {
                control.UpdateComboBoxItems(e.NewValue.ToString());
            }
        }

        private void UpdateComboBoxItems(string Items)
        {
            if (Items != null)
            {
                IEnumerable<string> items = Items.Split(',').Select(item => item.Trim());

                foreach (string item in items)
                {
                    ItemsSource.Add(item.Trim());
                }
            }
        }
    }
}