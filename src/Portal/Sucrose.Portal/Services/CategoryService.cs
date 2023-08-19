namespace Sucrose.Portal.Services
{
    internal class CategoryService : IDisposable
    {
        private string _CategoryTag = string.Empty;
        private string _BeforeCategoryTag = string.Empty;

        public event EventHandler CategoryTagChanged;
        public event EventHandler BeforeCategoryTagChanged;

        public string CategoryTag
        {
            get => _CategoryTag;
            set
            {
                if (_CategoryTag != value)
                {
                    BeforeCategoryTag = _CategoryTag;
                    _CategoryTag = value;
                    CategoryTagChanged?.Invoke(this, EventArgs.Empty);
                }
            }
        }

        public string BeforeCategoryTag
        {
            get => _BeforeCategoryTag;
            set
            {
                if (_BeforeCategoryTag != value)
                {
                    _BeforeCategoryTag = value;
                    BeforeCategoryTagChanged?.Invoke(this, EventArgs.Empty);
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