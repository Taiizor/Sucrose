using SPHL = Sucrose.Property.Helper.Localization;
using SPHP = Sucrose.Property.Helper.Properties;
using SSTMTBM = Sucrose.Shared.Theme.Model.TextBoxModel;
using ToolTip = System.Windows.Controls.ToolTip;
using UserControl = System.Windows.Controls.UserControl;

namespace Sucrose.Property.Controls
{
    /// <summary>
    /// TextBox.xaml etkileşim mantığı
    /// </summary>
    public partial class TextBox : UserControl
    {
        public TextBox(string Key, SSTMTBM Data)
        {
            InitializeComponent();

            InitializeData(Key, Data);
        }

        private void InitializeData(string Key, SSTMTBM Data)
        {
            Data.Text = SPHL.Convert(Data.Text);
            Data.Value = SPHL.Convert(Data.Value);

            Component.Text = Data.Value;
            Component.PlaceholderText = Data.Text;

            Component.TextChanged += (s, e) => Component_Changed(Key, Data, Component.Text);

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

        private void Component_Changed(string Key, SSTMTBM Data, string Value)
        {
            Data.Value = Value;

            SPHP.Change(Key, Data);
        }
    }
}