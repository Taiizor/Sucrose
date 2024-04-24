using SPHP = Sucrose.Property.Helper.Properties;
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
        public DropDown(string Key, SSTMDDM Data)
        {
            InitializeComponent();

            InitializeData(Key, Data);
        }

        private void InitializeData(string Key, SSTMDDM Data)
        {
            Label.Text = Data.Text;
            Component.ItemsSource = Data.Items;
            Component.SelectedIndex = Data.Value;

            Component.SelectionChanged += (s, e) => Component_Changed(Key, Data, Component.SelectedIndex);

            if (!string.IsNullOrEmpty(Data.Help))
            {
                ToolTip HelpTip = new()
                {
                    Content = Data.Help
                };

                Component.ToolTip = HelpTip;
            }
        }

        private void Component_Changed(string Key, SSTMDDM Data, int Value)
        {
            Data.Value = Value;

            SPHP.Change(Key, Data);
        }
    }
}