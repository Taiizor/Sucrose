using Avalonia.Styling;
using FluentAvalonia.Styling;
using ReactiveUI;
using System.Windows.Input;

namespace Sucrose.Portal.ViewModels;

public class MainViewModel : ViewModelBase
{
    public string Search => string.Empty;

    public string Greeting => "Welcome to Avalonia!";

    public ICommand DarkCommand { get; }

    public ICommand LightCommand { get; }

    public ICommand DefaultCommand { get; }


    private readonly FluentAvaloniaTheme _faTheme;

    public MainViewModel()
    {
        _faTheme = App.Current.Styles[0] as FluentAvaloniaTheme;

        DarkCommand = ReactiveCommand.CreateFromTask(async () =>
        {
            App.Current.RequestedThemeVariant = ThemeVariant.Dark;
        });

        LightCommand = ReactiveCommand.CreateFromTask(async () =>
        {
            App.Current.RequestedThemeVariant = ThemeVariant.Light;
        });

        DefaultCommand = ReactiveCommand.CreateFromTask(async () =>
        {
            App.Current.RequestedThemeVariant = ThemeVariant.Default;
        });
    }
}