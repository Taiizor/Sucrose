using System.Windows.Media;
using SMMM = Sucrose.Manager.Manage.Manager;
using SPMM = Sucrose.Portal.Manage.Manager;

namespace Sucrose.Portal.Services
{
    internal class BackdropService : IDisposable
    {
        private string _BackdropImage = SMMM.BackgroundImage;
        private int _BackdropOpacity = SMMM.BackgroundOpacity;
        private Stretch _BackdropStretch = SPMM.BackgroundStretch;

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