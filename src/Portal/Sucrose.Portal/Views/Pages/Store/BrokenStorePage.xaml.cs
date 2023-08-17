using System.Windows.Controls;

namespace Sucrose.Portal.Views.Pages.Store
{
    /// <summary>
    /// BrokenStorePage.xaml etkileşim mantığı
    /// </summary>
    public partial class BrokenStorePage : Page, IDisposable
    {
        public BrokenStorePage()
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