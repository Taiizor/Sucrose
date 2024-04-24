using SSTMPBM = Sucrose.Shared.Theme.Model.PasswordBoxModel;
using ToolTip = System.Windows.Controls.ToolTip;
using UserControl = System.Windows.Controls.UserControl;

namespace Sucrose.Property.Controls
{
    /// <summary>
    /// PasswordBox.xaml etkileşim mantığı
    /// </summary>
    public partial class PasswordBox : UserControl
    {
        public PasswordBox(SSTMPBM Data)
        {
            InitializeComponent();

            InitializeData(Data);
        }

        private void InitializeData(SSTMPBM Data)
        {
            Component.Password = Data.Value;
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