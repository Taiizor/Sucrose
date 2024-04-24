using SSTMCBM = Sucrose.Shared.Theme.Model.CheckBoxModel;
using ToolTip = System.Windows.Controls.ToolTip;
using UserControl = System.Windows.Controls.UserControl;

namespace Sucrose.Property.Controls
{
    /// <summary>
    /// CheckBox.xaml etkileşim mantığı
    /// </summary>
    public partial class CheckBox : UserControl
    {
        public CheckBox(SSTMCBM Data)
        {
            InitializeComponent();

            InitializeData(Data);
        }

        private void InitializeData(SSTMCBM Data)
        {
            Component.Content = Data.Text;
            Component.IsChecked = Data.Value;

            if (!string.IsNullOrEmpty(Data.Help))
            {
                ToolTip HelpTip = new()
                {
                    Content = Data.Help
                };

                Component.ToolTip = HelpTip;
            }
        }
    }
}