namespace Sucrose.Portal.Services
{
    internal class SearchService : IDisposable
    {
        private string _SearchText = string.Empty;
        private string[] _SearchList = Array.Empty<string>();

        private string _BeforeSearchText = string.Empty;
        private string[] _BeforeSearchList = Array.Empty<string>();

        public event EventHandler SearchTextChanged;
        public event EventHandler BeforeSearchTextChanged;

        public string SearchText
        {
            get => _SearchText.ToLowerInvariant();
            set
            {
                if (_SearchText != value.ToLowerInvariant())
                {
                    BeforeSearchText = _SearchText;
                    _SearchText = value.ToLowerInvariant();
                    SearchTextChanged?.Invoke(this, EventArgs.Empty);
                }
            }
        }

        public string[] SearchList => _SearchText.Trim().Replace("  ", " ").Split(' ');

        public string BeforeSearchText
        {
            get => _BeforeSearchText.ToLowerInvariant();
            set
            {
                if (_BeforeSearchText != value.ToLowerInvariant())
                {
                    _BeforeSearchText = value.ToLowerInvariant();
                    BeforeSearchTextChanged?.Invoke(this, EventArgs.Empty);
                }
            }
        }

        public string[] BeforeSearchList => _BeforeSearchText.Trim().Replace("  ", " ").Split(' ');

        public void Dispose()
        {
            GC.Collect();
            GC.SuppressFinalize(this);
        }
    }
}