using Microsoft.UI;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using Microsoft.UI.Xaml.Controls.Primitives;
using Microsoft.UI.Xaml.Data;
using Microsoft.UI.Xaml.Input;
using Microsoft.UI.Xaml.Media;
using Microsoft.UI.Xaml.Navigation;
using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Runtime.InteropServices.WindowsRuntime;
using Windows.Foundation;
using Windows.Foundation.Collections;
using WinUIEx;

// To learn more about WinUI, the WinUI project structure,
// and more about our project templates, see: http://aka.ms/winui-project-info.

namespace WinUIExSample.Pages
{
    /// <summary>
    /// An empty page that can be used on its own or navigated to within a Frame.
    /// </summary>
    public sealed partial class WindowDesign : Page
    {
        internal static MicaBackdrop micaBackdrop = new MicaBackdrop();
        private static DesktopAcrylicBackdrop acrylicBackdrop = new DesktopAcrylicBackdrop();
        private static TransparentTintBackdrop transparentTintBackdrop = new TransparentTintBackdrop();
        private static ColorAnimatedBackdrop colorRotatingBackdrop = new ColorAnimatedBackdrop();
        private static BlurredBackdrop blurredBackdrop = new BlurredBackdrop();
        private bool isInitialized;

        public WindowDesign()
        {
            this.InitializeComponent();
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

        public WindowEx MainWindow => ((App)Application.Current).MainWindow;

        private void Presenter_SelectionChanged(object sender, SelectionChangedEventArgs e)
        {
            if (!isInitialized) return;
            var index = ((ComboBox)sender).SelectedIndex;
            if (index == 0)
                MainWindow.PresenterKind = Microsoft.UI.Windowing.AppWindowPresenterKind.Overlapped;
            else if (index == 1)
                MainWindow.PresenterKind = Microsoft.UI.Windowing.AppWindowPresenterKind.CompactOverlay;
            else if (index == 2)
                MainWindow.PresenterKind = Microsoft.UI.Windowing.AppWindowPresenterKind.FullScreen;
        }



        private void Backdrop_SelectionChanged(object sender, SelectionChangedEventArgs e)
        {
            if (!isInitialized) return;
            switch (((ComboBox)sender).SelectedIndex)
            {
                case 1: MainWindow.SystemBackdrop = acrylicBackdrop; break;
                case 2: MainWindow.SystemBackdrop = transparentTintBackdrop; break;
                case 3: MainWindow.SystemBackdrop = colorRotatingBackdrop; break;
                case 4: MainWindow.SystemBackdrop = blurredBackdrop; break;
                default: MainWindow.SystemBackdrop = micaBackdrop; break;
            }
        }

        private class ColorAnimatedBackdrop : CompositionBrushBackdrop
        {
            protected override Windows.UI.Composition.CompositionBrush CreateBrush(Windows.UI.Composition.Compositor compositor)
            {
                var brush = compositor.CreateColorBrush(Windows.UI.Color.FromArgb(255, 255, 0, 0));
                var animation = compositor.CreateColorKeyFrameAnimation();
                var easing = compositor.CreateLinearEasingFunction();
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
                => compositor.CreateHostBackdropBrush();
        }
    }
}