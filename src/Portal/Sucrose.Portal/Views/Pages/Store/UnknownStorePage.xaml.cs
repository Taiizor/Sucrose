using System.Windows.Controls;

namespace Sucrose.Portal.Views.Pages.Store
{
    /// <summary>
    /// UnknownStorePage.xaml etkileşim mantığı
    /// </summary>
    public partial class UnknownStorePage : Page, IDisposable
    {
        public UnknownStorePage()
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