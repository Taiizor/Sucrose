using System.Windows.Controls;

namespace Sucrose.Portal.Views.Pages.Store
{
    /// <summary>
    /// SearchStorePage.xaml etkileşim mantığı
    /// </summary>
    public partial class SearchStorePage : Page, IDisposable
    {
        public SearchStorePage()
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