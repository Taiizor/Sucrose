namespace Sucrose.Portal.Services
{
    internal class CultureService : IDisposable
    {
        private string _CultureCode = string.Empty;
        private string _BeforeCultureCode = string.Empty;

        public event EventHandler CultureCodeChanged;
        public event EventHandler BeforeCultureCodeChanged;

        public string CultureCode
        {
            get => _CultureCode;
            set
            {
                if (_CultureCode != value)
                {
                    BeforeCultureCode = _CultureCode;
                    _CultureCode = value;
                    CultureCodeChanged?.Invoke(this, EventArgs.Empty);
                }
            }
        }

        public string BeforeCultureCode
        {
            get => _BeforeCultureCode;
            set
            {
                if (_BeforeCultureCode != value)
                {
                    _BeforeCultureCode = value;
                    BeforeCultureCodeChanged?.Invoke(this, EventArgs.Empty);
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