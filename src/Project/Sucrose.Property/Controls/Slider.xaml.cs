using SSTMSM = Sucrose.Shared.Theme.Model.SliderModel;
using ToolTip = System.Windows.Controls.ToolTip;
using UserControl = System.Windows.Controls.UserControl;

namespace Sucrose.Property.Controls
{
    /// <summary>
    /// Slider.xaml etkileşim mantığı
    /// </summary>
    public partial class Slider : UserControl
    {
        public Slider(SSTMSM Data)
        {
            InitializeComponent();

            InitializeData(Data);
        }

        private void InitializeData(SSTMSM Data)
        {
            Component.Maximum = Data.Max;
            Component.Minimum = Data.Min;
            Component.Value = Data.Value;
            Component.TickFrequency = Data.Step;

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