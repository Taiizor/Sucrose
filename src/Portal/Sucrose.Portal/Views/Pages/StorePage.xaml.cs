
namespace Sucrose.Portal.Views.Pages
{
    /// <summary>
    /// StorePage.xaml etkileşim mantığı
    /// </summary>
    public partial class StorePage : IDisposable
    {
        public StorePage()
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