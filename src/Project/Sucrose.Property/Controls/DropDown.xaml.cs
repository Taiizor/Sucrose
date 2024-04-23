using UserControl = System.Windows.Controls.UserControl;

namespace Sucrose.Property.Controls
{
    /// <summary>
    /// DropDown.xaml etkileşim mantığı
    /// </summary>
    public partial class DropDown : UserControl
    {
        public DropDown()
        {
            DataContext = this;
            InitializeComponent();
        }
    }
}