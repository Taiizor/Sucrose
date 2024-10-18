using System.Windows.Media;
using SMMP = Sucrose.Manager.Manage.Portal;
using SSDMMP = Sucrose.Shared.Dependency.Manage.Manager.Portal;

namespace Sucrose.Portal.Services
{
    internal class BackdropService : IDisposable
    {
        public event EventHandler BackdropImageChanged;
        public event EventHandler BackdropStretchChanged;
        public event EventHandler BackdropOpacityChanged;

        public string BackdropImage
        {
            get;
            set
            {
                if (field != value)
                {
                    field = value;
                    BackdropImageChanged?.Invoke(this, EventArgs.Empty);
                }
            }
        } = SMMP.BackgroundImage;

        public Stretch BackdropStretch
        {
            get;
            set
            {
                if (field != value)
                {
                    field = value;
                    BackdropStretchChanged?.Invoke(this, EventArgs.Empty);
                }
            }
        } = SSDMMP.BackgroundStretch;

        public int BackdropOpacity
        {
            get;
            set
            {
                if (field != value)
                {
                    field = value;
                    BackdropOpacityChanged?.Invoke(this, EventArgs.Empty);
                }
            }
        } = SMMP.BackgroundOpacity;

        public void Dispose()
        {
            GC.Collect();
            GC.SuppressFinalize(this);
        }
    }
}