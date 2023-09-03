using System.Windows;

namespace Sucrose.Portal.Services
{
    internal class DonateService : IDisposable
    {
        private Visibility _DonateVisibility;

        public event EventHandler DonateVisibilityChanged;

        public Visibility DonateVisibility
        {
            get => _DonateVisibility;
            set
            {
                if (_DonateVisibility != value)
                {
                    _DonateVisibility = value;
                    DonateVisibilityChanged?.Invoke(this, EventArgs.Empty);
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