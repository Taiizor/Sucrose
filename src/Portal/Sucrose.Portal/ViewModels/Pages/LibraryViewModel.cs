namespace Sucrose.Portal.ViewModels.Pages
{
    public partial class LibraryViewModel : ViewModel, IDisposable
    {
        public void Dispose()
        {
            GC.Collect();
            GC.SuppressFinalize(this);
        }
    }
}