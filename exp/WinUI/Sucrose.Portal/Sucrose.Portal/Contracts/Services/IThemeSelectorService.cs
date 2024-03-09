using Microsoft.UI.Xaml;

namespace Sucrose.Portal.Contracts.Services;

public interface IThemeSelectorService
{
    ElementTheme Theme
    {
        get;
    }

    Task SetThemeAsync(ElementTheme theme);

    Task SetRequestedThemeAsync();
}
