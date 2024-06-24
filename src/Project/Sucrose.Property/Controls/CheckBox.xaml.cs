using SPHL = Sucrose.Property.Helper.Localization;
using SPHP = Sucrose.Property.Helper.Properties;
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
        public CheckBox(string Key, SSTMCBM Data)
        {
            InitializeComponent();

            InitializeData(Key, Data);
        }

        private void InitializeData(string Key, SSTMCBM Data)
        {
            Data.Text = SPHL.Convert(Data.Text);

            Component.Content = Data.Text;
            Component.IsChecked = Data.Value;

            Component.Checked += (s, e) => Component_Changed(Key, Data, true);
            Component.Unchecked += (s, e) => Component_Changed(Key, Data, false);

            if (!string.IsNullOrEmpty(Data.Help))
            {
                Data.Help = SPHL.Convert(Data.Help);

                ToolTip HelpTip = new()
                {
                    Content = Data.Help
                };

                Component.ToolTip = HelpTip;
            }
        }

        private void Component_Changed(string Key, SSTMCBM Data, bool Value)
        {
            Data.Value = Value;

            SPHP.Change(Key, Data);
        }
    }
}