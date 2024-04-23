using UserControl = System.Windows.Controls.UserControl;

namespace Sucrose.Property.Controls
{
    /// <summary>
    /// FileDropDown.xaml etkileşim mantığı
    /// </summary>
    public partial class FileDropDown : UserControl
    {
        public FileDropDown()
        {
            DataContext = this;
            InitializeComponent();
        }
    }
}