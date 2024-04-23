using SSTMLM = Sucrose.Shared.Theme.Model.LabelModel;
using UserControl = System.Windows.Controls.UserControl;
using ToolTip = System.Windows.Controls.ToolTip;

namespace Sucrose.Property.Controls
{
    /// <summary>
    /// Label.xaml etkileşim mantığı
    /// </summary>
    public partial class Label : UserControl
    {
        public Label(SSTMLM Data)
        {
            DataContext = this;

            InitializeComponent();

            InitializeData(Data);
        }

        private void InitializeData(SSTMLM Data)
        {
            Component.Text = Data.Value;

            ToolTip HelpTip = new()
            {
                Content = Data.Help
            };

            Component.ToolTip = HelpTip;
        }
    }
}