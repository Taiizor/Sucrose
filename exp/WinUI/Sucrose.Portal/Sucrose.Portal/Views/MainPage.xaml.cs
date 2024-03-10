using Microsoft.UI.Xaml.Controls;
using Sucrose.Portal.ViewModels;
using static Sucrose.Portal.MainWindow;

namespace Sucrose.Portal.Views;

public sealed partial class MainPage : Page
{
    private bool isInitialized;

    public MainViewModel ViewModel
    {
        get;
    }

    public MainPage()
    {
        ViewModel = App.GetService<MainViewModel>();

        InitializeComponent();

        backdropSelector.SelectedIndex = 0;

        presenter.SelectedIndex = MainWindow.PresenterKind switch
        {
            Microsoft.UI.Windowing.AppWindowPresenterKind.Overlapped => 0,
            Microsoft.UI.Windowing.AppWindowPresenterKind.CompactOverlay => 1,
            Microsoft.UI.Windowing.AppWindowPresenterKind.FullScreen => 2,
            _ => 0
        };

        isInitialized = true;
    }

    public WindowEx MainWindow => App.MainWindow;

    private void Presenter_SelectionChanged(object sender, SelectionChangedEventArgs e)
    {
        if (!isInitialized)
        {
            return;
        }

        int index = ((ComboBox)sender).SelectedIndex;

        if (index == 0)
        {
            MainWindow.PresenterKind = Microsoft.UI.Windowing.AppWindowPresenterKind.Overlapped;
        }
        else if (index == 1)
        {
            MainWindow.PresenterKind = Microsoft.UI.Windowing.AppWindowPresenterKind.CompactOverlay;
        }
        else if (index == 2)
        {
            MainWindow.PresenterKind = Microsoft.UI.Windowing.AppWindowPresenterKind.FullScreen;
        }
    }

    private void Backdrop_SelectionChanged(object sender, SelectionChangedEventArgs e)
    {
        if (!isInitialized)
        {
            return;
        }

        switch ((BackdropType)((ComboBox)sender).SelectedIndex)
        {
            case BackdropType.Mica:
                (MainWindow as MainWindow).SetBackdrop(BackdropType.Mica);
                break;
            case BackdropType.Blur:
                (MainWindow as MainWindow).SetBackdrop(BackdropType.Blur);
                break;
            case BackdropType.MicaAlt:
                (MainWindow as MainWindow).SetBackdrop(BackdropType.MicaAlt);
                break;
            case BackdropType.Animated:
                (MainWindow as MainWindow).SetBackdrop(BackdropType.Animated);
                break;
            case BackdropType.Transparent:
                (MainWindow as MainWindow).SetBackdrop(BackdropType.Transparent);
                break;
            case BackdropType.DesktopAcrylic:
                (MainWindow as MainWindow).SetBackdrop(BackdropType.DesktopAcrylic);
                break;
            case BackdropType.DesktopAcrylicBase:
                (MainWindow as MainWindow).SetBackdrop(BackdropType.DesktopAcrylicBase);
                break;
            case BackdropType.DesktopAcrylicThin:
                (MainWindow as MainWindow).SetBackdrop(BackdropType.DesktopAcrylicThin);
                break;
            default:
                break;
        }
    }
}