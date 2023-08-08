using System.Windows.Controls;

namespace Sucrose.Portal.Views.Pages.Library
{
    /// <summary>
    /// EmptyLibraryPage.xaml etkileşim mantığı
    /// </summary>
    public partial class EmptyLibraryPage : Page, IDisposable
    {
        public EmptyLibraryPage()
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