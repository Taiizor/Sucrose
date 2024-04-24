using SPHP = Sucrose.Property.Helper.Properties;
using SSTMNBM = Sucrose.Shared.Theme.Model.NumberBoxModel;
using ToolTip = System.Windows.Controls.ToolTip;
using UserControl = System.Windows.Controls.UserControl;

namespace Sucrose.Property.Controls
{
    /// <summary>
    /// NumberBox.xaml etkileşim mantığı
    /// </summary>
    public partial class NumberBox : UserControl
    {
        public NumberBox(string Key, SSTMNBM Data)
        {
            InitializeComponent();

            InitializeData(Key, Data);
        }

        private void InitializeData(string Key, SSTMNBM Data)
        {
            Component.Maximum = Data.Max;
            Component.Minimum = Data.Min;
            Component.Value = Data.Value;
            Component.PlaceholderText = Data.Text;
            Component.MaxDecimalPlaces = Data.Places;

            Component.ValueChanged += (s, e) => Component_Changed(Key, Data, Component.Value);

            if (!string.IsNullOrEmpty(Data.Help))
            {
                ToolTip HelpTip = new()
                {
                    Content = Data.Help
                };

                Component.ToolTip = HelpTip;
            }
        }

        private void Component_Changed(string Key, SSTMNBM Data, double? Value)
        {
            if (Value == null)
            {
                Data.Value = 0;
                Component.Value = 0;
            }
            else
            {
                Data.Value = (double)Value;
            }

            SPHP.Change(Key, Data);
        }
    }
}