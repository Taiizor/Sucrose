using Microsoft.UI;
using Microsoft.UI.Xaml.Controls;
using Microsoft.UI.Xaml.Media;
using Sucrose.Portal.ViewModels;

namespace Sucrose.Portal.Views;

public sealed partial class MainPage : Page
{
    internal static MicaBackdrop micaBackdrop = new();
    private static DesktopAcrylicBackdrop acrylicBackdrop = new();
    private static TransparentTintBackdrop transparentTintBackdrop = new();
    private static ColorAnimatedBackdrop colorRotatingBackdrop = new();
    private static BlurredBackdrop blurredBackdrop = new();

    private bool isInitialized;

    public MainViewModel ViewModel
    {
        get;
    }

    public MainPage()
    {
        ViewModel = App.GetService<MainViewModel>();

        InitializeComponent();

        isInitialized = true;

        backdropSelector.SelectedIndex = MainWindow.SystemBackdrop switch
        {
            DesktopAcrylicBackdrop => 1,
            TransparentTintBackdrop => 2,
            ColorAnimatedBackdrop => 3,
            BlurredBackdrop => 4,
            _ => 0
        };

        presenter.SelectedIndex = MainWindow.PresenterKind switch
        {
            Microsoft.UI.Windowing.AppWindowPresenterKind.Overlapped => 0,
            Microsoft.UI.Windowing.AppWindowPresenterKind.CompactOverlay => 1,
            Microsoft.UI.Windowing.AppWindowPresenterKind.FullScreen => 2,
            _ => 0
        };
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

        MainWindow.SystemBackdrop = ((ComboBox)sender).SelectedIndex switch
        {
            1 => acrylicBackdrop,
            2 => transparentTintBackdrop,
            3 => colorRotatingBackdrop,
            4 => blurredBackdrop,
            _ => micaBackdrop,
        };
    }

    private class ColorAnimatedBackdrop : CompositionBrushBackdrop
    {
        protected override Windows.UI.Composition.CompositionBrush CreateBrush(Windows.UI.Composition.Compositor compositor)
        {
            Windows.UI.Composition.CompositionColorBrush brush = compositor.CreateColorBrush(Windows.UI.Color.FromArgb(255, 255, 0, 0));
            Windows.UI.Composition.ColorKeyFrameAnimation animation = compositor.CreateColorKeyFrameAnimation();
            Windows.UI.Composition.LinearEasingFunction easing = compositor.CreateLinearEasingFunction();

            animation.InsertKeyFrame(0, Colors.Red, easing);
            animation.InsertKeyFrame(.333f, Colors.Green, easing);
            animation.InsertKeyFrame(.667f, Colors.Blue, easing);
            animation.InsertKeyFrame(1, Colors.Red, easing);

            animation.InterpolationColorSpace = Windows.UI.Composition.CompositionColorSpace.Hsl;

            animation.Duration = TimeSpan.FromSeconds(15);

            animation.IterationBehavior = Windows.UI.Composition.AnimationIterationBehavior.Forever;

            brush.StartAnimation("Color", animation);

            return brush;
        }
    }

    private class BlurredBackdrop : CompositionBrushBackdrop
    {
        protected override Windows.UI.Composition.CompositionBrush CreateBrush(Windows.UI.Composition.Compositor compositor)
        {
            return compositor.CreateHostBackdropBrush();
        }
    }
}