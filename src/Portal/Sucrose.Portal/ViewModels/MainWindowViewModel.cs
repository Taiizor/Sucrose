using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;

namespace Sucrose.Portal.ViewModels
{
    public partial class MainWindowViewModel : ObservableObject
    {
        [RelayCommand]
        public void OnMenuAction(string parameter) { }
    }
}