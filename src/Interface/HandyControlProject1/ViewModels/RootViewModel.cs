using Stylet;

namespace HandyControlProject1.ViewModels
{
    public class RootViewModel : PropertyChangedBase
    {
        private string _title = "HandyControl Application";
        public string Title
        {
            get => _title;
            set => SetAndNotify(ref _title, value);
        }

        public RootViewModel()
        {

        }
    }
}
