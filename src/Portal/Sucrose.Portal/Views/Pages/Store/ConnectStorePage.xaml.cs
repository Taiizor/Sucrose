using System.Windows.Controls;

namespace Sucrose.Portal.Views.Pages.Store
{
    /// <summary>
    /// ConnectStorePage.xaml etkileşim mantığı
    /// </summary>
    public partial class ConnectStorePage : Page, IDisposable
    {
        public ConnectStorePage()
        {
            InitializeComponent();
        }

        public void Dispose()
        {
            GC.Collect();
            GC.SuppressFinalize(this);
        }
    }
}