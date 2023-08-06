using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using Wpf.Ui.Common;
using Sucrose.Portal.Models;
using Sucrose.Portal.Services;

namespace Sucrose.Portal.ViewModels
{
    public partial class LibraryViewModel : ObservableObject
    {
        private readonly WindowsProviderService _windowsProviderService;

        [ObservableProperty]
        private IEnumerable<WindowCard> _windowCards = new WindowCard[]
        {
        new("Monaco", "Visual Studio Code in your WPF app.", SymbolRegular.CodeBlock24, "monaco"),
        new("Editor", "Text editor with tabbed background.", SymbolRegular.ScanText24, "editor")
        };

        public LibraryViewModel(WindowsProviderService windowsProviderService)
        {
            _windowsProviderService = windowsProviderService;
        }

        [RelayCommand]
        public void OnOpenWindow(string value)
        {
            if (String.IsNullOrEmpty(value))
                return;

            switch (value)
            {
                case "monaco":
                    //
                    break;

                case "editor":
                    //
                    break;
            }
        }
    }
}