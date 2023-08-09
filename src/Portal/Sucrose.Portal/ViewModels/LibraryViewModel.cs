using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using Wpf.Ui.Controls;

namespace Sucrose.Portal.ViewModels
{
    public partial class LibraryViewModel : ObservableObject, INavigationAware, IDisposable
    {
        [ObservableProperty]
        private int _counter = 0;

        [ObservableProperty]
        private string _Test = "Testing";

        public void OnNavigatedTo()
        {
        }

        public void OnNavigatedFrom()
        {
        }

        [RelayCommand]
        private void OnCounterIncrement()
        {
            Counter++;
        }

        public void Dispose()
        {
            GC.Collect();
            GC.SuppressFinalize(this);
        }
    }
}
