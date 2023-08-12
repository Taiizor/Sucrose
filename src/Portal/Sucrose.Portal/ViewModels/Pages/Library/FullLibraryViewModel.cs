using CommunityToolkit.Mvvm.ComponentModel;
using Wpf.Ui.Controls;

namespace Sucrose.Portal.ViewModels.Pages.Library
{
    public partial class FullLibraryViewModel : ObservableObject, INavigationAware, IDisposable
    {
        public void OnNavigatedTo()
        {
        }

        public void OnNavigatedFrom()
        {
        }

        public void Dispose()
        {
            GC.Collect();
            GC.SuppressFinalize(this);
        }
    }
}