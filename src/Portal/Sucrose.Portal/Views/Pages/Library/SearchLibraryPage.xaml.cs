using System.Windows.Controls;

namespace Sucrose.Portal.Views.Pages.Library
{
    /// <summary>
    /// SearchLibraryPage.xaml etkileşim mantığı
    /// </summary>
    public partial class SearchLibraryPage : Page, IDisposable
    {
        public SearchLibraryPage()
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