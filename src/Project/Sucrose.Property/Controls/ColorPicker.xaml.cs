using HandyControl.Controls;
using HandyControl.Tools;
using System.Windows;
using Control = System.Windows.Controls.Control;
using CPicker = HandyControl.Controls.ColorPicker;
using UserControl = System.Windows.Controls.UserControl;

namespace Sucrose.Property.Controls
{
    /// <summary>
    /// ColorPicker.xaml etkileşim mantığı
    /// </summary>
    public partial class ColorPicker : UserControl
    {
        public Control Control { get; set; }

        public ColorPicker()
        {
            DataContext = this;
            InitializeComponent();
        }

        private void DropDownButton_Click(object sender, RoutedEventArgs e)
        {
            CPicker picker = SingleOpenHelper.CreateControl<CPicker>();

            System.Windows.Media.Color Temp = SCB.Color;

            picker.SelectedBrush = new(Temp);
            picker.UseLayoutRounding = true;

            PopupWindow window = new()
            {
                PopupElement = picker,
                WindowStartupLocation = WindowStartupLocation.Manual,
                AllowsTransparency = true,
                WindowStyle = WindowStyle.None,
                ResizeMode = ResizeMode.NoResize,
                UseLayoutRounding = true,
                ShowBorder = true,
                Topmost = true,
                Title = "Sucrose Property Color Picker"
            };

            picker.Confirmed += (s, ee) => { SCB.Color = ee.Info; window.Close(); };
            picker.SelectedColorChanged += (s, ee) => { SCB.Color = ee.Info; };
            picker.Canceled += delegate { SCB.Color = Temp; window.Close(); };

            window.Show(Control, false);
        }
    }
}