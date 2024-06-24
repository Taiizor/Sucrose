using SPHL = Sucrose.Property.Helper.Localization;
using SPHP = Sucrose.Property.Helper.Properties;
using SSTMBM = Sucrose.Shared.Theme.Model.ButtonModel;
using ToolTip = System.Windows.Controls.ToolTip;
using UserControl = System.Windows.Controls.UserControl;

namespace Sucrose.Property.Controls
{
    /// <summary>
    /// Button.xaml etkileşim mantığı
    /// </summary>
    public partial class Button : UserControl
    {
        public Button(string Key, SSTMBM Data)
        {
            InitializeComponent();

            InitializeData(Key, Data);
        }

        private void InitializeData(string Key, SSTMBM Data)
        {
            Data.Text = SPHL.Convert(Data.Text);

            Component.Content = Data.Text;

            Component.Click += (s, e) => Component_Click(Key, Data);

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

        private void Component_Click(string Key, SSTMBM Data)
        {
            SPHP.Change(Key, Data);
        }
    }
}