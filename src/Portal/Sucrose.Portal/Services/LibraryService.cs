namespace Sucrose.Portal.Services
{
    internal class LibraryService : IDisposable
    {
        public string Theme = string.Empty;

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