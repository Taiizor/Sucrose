using System.Windows.Media;

namespace Sucrose.Portal.Services
{
    internal class BackdropService : IDisposable
    {
        private int _BackdropOpacity;
        private string _BackdropImage;
        private Stretch _BackdropStretch;

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