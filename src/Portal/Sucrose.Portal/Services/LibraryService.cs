namespace Sucrose.Portal.Services
{
    internal class LibraryService : IDisposable
    {
        public event EventHandler CreatedWallpaper;

        public void CreateWallpaper()
        {
            CreatedWallpaper?.Invoke(this, EventArgs.Empty);
        }

        public void Dispose()
        {
            GC.Collect();
            GC.SuppressFinalize(this);
        }
    }
}