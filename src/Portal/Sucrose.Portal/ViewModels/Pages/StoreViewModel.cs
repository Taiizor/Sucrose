namespace Sucrose.Portal.ViewModels.Pages
{
    public partial class StoreViewModel : ViewModel, IDisposable
    {
        public void Dispose()
        {
            GC.Collect();
            GC.SuppressFinalize(this);
        }
    }
}