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
        public TextBox(SSTMTBM Data)
        {
            InitializeComponent();

            InitializeData(Data);
        }

        private void InitializeData(SSTMTBM Data)
        {
            Component.Text = Data.Value;
            Component.PlaceholderText = Data.Text;

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