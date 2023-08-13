namespace Sucrose.Portal.Services
{
    internal class SearchService
    {
        private string _SearchText = string.Empty;
        private string _BeforeSearchText = string.Empty;

        public event EventHandler SearchTextChanged;

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

        public string BeforeSearchText => _BeforeSearchText.ToLowerInvariant();
    }
}