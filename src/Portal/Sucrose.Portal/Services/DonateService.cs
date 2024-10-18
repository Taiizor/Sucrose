using System.Windows;

namespace Sucrose.Portal.Services
{
    internal class DonateService : IDisposable
    {
        public event EventHandler DonateVisibilityChanged;

        public Visibility DonateVisibility
        {
            get;
            set
            {
                if (field != value)
                {
                    field = value;
                    DonateVisibilityChanged?.Invoke(this, EventArgs.Empty);
                }
            }
        } = Visibility.Hidden;

        public void Dispose()
        {
            GC.Collect();
            GC.SuppressFinalize(this);
        }
    }
}