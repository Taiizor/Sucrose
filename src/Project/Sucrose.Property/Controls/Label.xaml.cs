using SPHL = Sucrose.Property.Helper.Localization;
using SSTMLM = Sucrose.Shared.Theme.Model.LabelModel;
using ToolTip = System.Windows.Controls.ToolTip;
using UserControl = System.Windows.Controls.UserControl;

namespace Sucrose.Property.Controls
{
    /// <summary>
    /// Label.xaml etkileşim mantığı
    /// </summary>
    public partial class Label : UserControl
    {
        public Label(SSTMLM Data)
        {
            InitializeComponent();

            InitializeData(Data);
        }

        private void InitializeData(SSTMLM Data)
        {
            Data.Value = SPHL.Convert(Data.Value);

            Component.Text = Data.Value;

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
    }
}