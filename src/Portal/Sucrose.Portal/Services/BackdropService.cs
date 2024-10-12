using System.Windows.Media;
using SMMP = Sucrose.Manager.Manage.Portal;
using SSDMMP = Sucrose.Shared.Dependency.Manage.Manager.Portal;

namespace Sucrose.Portal.Services
{
    internal class BackdropService : IDisposable
    {
        private string _BackdropImage = SMMP.BackgroundImage;
        private int _BackdropOpacity = SMMP.BackgroundOpacity;
        private Stretch _BackdropStretch = SSDMMP.BackgroundStretch;

        public event EventHandler BackdropImageChanged;
        public event EventHandler BackdropStretchChanged;
        public event EventHandler BackdropOpacityChanged;

        public string BackdropImage
        {
            get => _BackdropImage;
            set
            {
                if (_BackdropImage != value)
                {
                    _BackdropImage = value;
                    BackdropImageChanged?.Invoke(this, EventArgs.Empty);
                }
            }
        }

        public Stretch BackdropStretch
        {
            get => _BackdropStretch;
            set
            {
                if (_BackdropStretch != value)
                {
                    _BackdropStretch = value;
                    BackdropStretchChanged?.Invoke(this, EventArgs.Empty);
                }
            }
        }

        public int BackdropOpacity
        {
            get => _BackdropOpacity;
            set
            {
                if (_BackdropOpacity != value)
                {
                    _BackdropOpacity = value;
                    BackdropOpacityChanged?.Invoke(this, EventArgs.Empty);
                }
            }
        }

        public void Dispose()
        {
            GC.Collect();
            GC.SuppressFinalize(this);
        }
    }
}