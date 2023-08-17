namespace Sucrose.Portal.Services
{
    internal class SearchService : IDisposable
    {
        private string _SearchText = string.Empty;
        private string _BeforeSearchText = string.Empty;

        public event EventHandler SearchTextChanged;
        public event EventHandler BeforeSearchTextChanged;

        public string SearchText
        {
            get => _SearchText.ToLowerInvariant();
            set
            {
                _BeforeSearchText = _SearchText;
                _SearchText = value.ToLowerInvariant();
                SearchTextChanged?.Invoke(this, EventArgs.Empty);
            }
        }

        public string BeforeSearchText
        {
            get => _BeforeSearchText.ToLowerInvariant();
            set
            {
                _BeforeSearchText = value.ToLowerInvariant();
                BeforeSearchTextChanged?.Invoke(this, EventArgs.Empty);
            }
        }

        public void Dispose()
        {
            GC.Collect();
            GC.SuppressFinalize(this);
        }
    }
}