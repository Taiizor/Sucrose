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
        public Button(SSTMBM Data)
        {
            InitializeComponent();

            InitializeData(Data);
        }

        private void InitializeData(SSTMBM Data)
        {
            Component.Content = Data.Text;

            Component.Click += async (s, e) => await Component_Click(Data);

            if (!string.IsNullOrEmpty(Data.Help))
            {
                ToolTip HelpTip = new()
                {
                    Content = Data.Help
                };

                Component.ToolTip = HelpTip;
            }
        }

        // Tıklama gerçekleştikten sonra yeni ayarlar cache property'sine aktarılır. Sonrasında tekrar komut gerçekleşmesin
        // diye değer null yapılır.
        private async Task Component_Click(SSTMBM Data)
        {
            Component.IsEnabled = false;

            Data.Value = Data.Command;

            await Task.Delay(100);

            Data.Value = null;

            await Task.Delay(100);

            Component.IsEnabled = true;

            await Task.CompletedTask;
        }
    }
}