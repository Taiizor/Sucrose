namespace Sucrose.Portal.Services
{
    internal class CultureService : IDisposable
    {
        public event EventHandler CultureCodeChanged;
        public event EventHandler BeforeCultureCodeChanged;

        public string CultureCode
        {
            get;
            set
            {
                if (field != value)
                {
                    BeforeCultureCode = field;
                    field = value;
                    CultureCodeChanged?.Invoke(this, EventArgs.Empty);
                }
            }
        } = string.Empty;

        public string BeforeCultureCode
        {
            get;
            set
            {
                if (field != value)
                {
                    field = value;
                    BeforeCultureCodeChanged?.Invoke(this, EventArgs.Empty);
                }
            }
        } = string.Empty;

        public void Dispose()
        {
            GC.Collect();
            GC.SuppressFinalize(this);
        }
    }
}