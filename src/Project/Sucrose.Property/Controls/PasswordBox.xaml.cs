using SPHL = Sucrose.Property.Helper.Localization;
using SPHP = Sucrose.Property.Helper.Properties;
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
        public PasswordBox(string Key, SSTMPBM Data)
        {
            InitializeComponent();

            InitializeData(Key, Data);
        }

        private void InitializeData(string Key, SSTMPBM Data)
        {
            Data.Text = SPHL.Convert(Data.Text);
            Data.Value = SPHL.Convert(Data.Value);

            Component.Password = Data.Value;
            Component.PlaceholderText = Data.Text;

            Component.PasswordChanged += (s, e) => Component_Changed(Key, Data, Component.Password);

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

        private void Component_Changed(string Key, SSTMPBM Data, string Value)
        {
            Data.Value = Value;

            SPHP.Change(Key, Data);
        }
    }
}