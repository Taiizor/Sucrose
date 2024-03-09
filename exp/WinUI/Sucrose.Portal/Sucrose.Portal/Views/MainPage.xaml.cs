using Microsoft.UI.Xaml.Controls;

using Sucrose.Portal.ViewModels;

namespace Sucrose.Portal.Views;

public sealed partial class MainPage : Page
{
    public MainViewModel ViewModel
    {
        get;
    }

    public MainPage()
    {
        ViewModel = App.GetService<MainViewModel>();
        InitializeComponent();
    }
}