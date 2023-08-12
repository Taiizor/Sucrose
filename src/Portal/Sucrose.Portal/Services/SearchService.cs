namespace Sucrose.Portal.Services
{
    internal class SearchService
    {
        public event EventHandler SearchTextChanged;

        private string _SearchText = string.Empty;

        public string SearchText
        {
            get => _SearchText.ToLowerInvariant();
            set
            {
                _SearchText = value.ToLowerInvariant();
                SearchTextChanged?.Invoke(this, EventArgs.Empty);
            }
        }
    }
}