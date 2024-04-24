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
        public NumberBox(SSTMNBM Data)
        {
            InitializeComponent();

            InitializeData(Data);
        }

        private void InitializeData(SSTMNBM Data)
        {
            Component.Maximum = Data.Max;
            Component.Minimum = Data.Min;
            Component.Value = Data.Value;
            Component.PlaceholderText = Data.Text;
            Component.MaxDecimalPlaces = Data.Places;

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