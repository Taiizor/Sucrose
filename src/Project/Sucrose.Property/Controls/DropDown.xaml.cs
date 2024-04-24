using SSTMDDM = Sucrose.Shared.Theme.Model.DropDownModel;
using ToolTip = System.Windows.Controls.ToolTip;
using UserControl = System.Windows.Controls.UserControl;

namespace Sucrose.Property.Controls
{
    /// <summary>
    /// DropDown.xaml etkileşim mantığı
    /// </summary>
    public partial class DropDown : UserControl
    {
        public DropDown(SSTMDDM Data)
        {
            InitializeComponent();

            InitializeData(Data);
        }

        private void InitializeData(SSTMDDM Data)
        {
            Component.Text = Data.Text;
            Component.ItemsSource = Data.Items;
            Component.SelectedIndex = Data.Value;

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