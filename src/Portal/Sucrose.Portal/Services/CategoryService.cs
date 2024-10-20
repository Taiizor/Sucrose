namespace Sucrose.Portal.Services
{
    internal class CategoryService : IDisposable
    {
        public event EventHandler CategoryTagChanged;
        public event EventHandler BeforeCategoryTagChanged;

        public string CategoryTag
        {
            get;
            set
            {
                if (field != value)
                {
                    field = value;
                    BeforeCategoryTag = field;
                    CategoryTagChanged?.Invoke(this, EventArgs.Empty);
                }
            }
        } = string.Empty;

        public string BeforeCategoryTag
        {
            get;
            set
            {
                if (field != value)
                {
                    field = value;
                    BeforeCategoryTagChanged?.Invoke(this, EventArgs.Empty);
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