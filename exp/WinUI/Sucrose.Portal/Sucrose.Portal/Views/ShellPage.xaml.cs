using Microsoft.UI.Input;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using Microsoft.UI.Xaml.Input;
using Microsoft.UI.Xaml.Media;
using Sucrose.Portal.Contracts.Services;
using Sucrose.Portal.Helpers;
using Sucrose.Portal.ViewModels;
using Windows.ApplicationModel.DataTransfer;
using Windows.Foundation;
using Windows.Graphics;
using Windows.System;

namespace Sucrose.Portal.Views;

// TODO: Update NavigationViewItem titles and icons in ShellPage.xaml.
public sealed partial class ShellPage : Page
{
    public ShellViewModel ViewModel
    {
        get;
    }

    public ShellPage(ShellViewModel viewModel)
    {
        ViewModel = viewModel;
        InitializeComponent();

        ViewModel.NavigationService.Frame = NavigationFrame;
        ViewModel.NavigationViewService.Initialize(NavigationViewControl);

        // TODO: Set the title bar icon by updating /Assets/WindowIcon.ico.
        // A custom title bar is required for full window theme and Mica support.
        // https://docs.microsoft.com/windows/apps/develop/title-bar?tabs=winui3#full-customization
        App.MainWindow.ExtendsContentIntoTitleBar = true;

        AppTitleBar.Loaded += AppTitleBar_Loaded;
        App.MainWindow.Activated += MainWindow_Activated;
        AppTitleBar.SizeChanged += AppTitleBar_SizeChanged;
        TitleBarSearchBox.SizeChanged += TitleBarSearchBox_SizeChanged;

        App.MainWindow.SetTitleBar(AppTitleBar);

        AppTitleBarText.Text = "AppDisplayName".GetLocalized();
    }

    private void AppTitleBar_Loaded(object sender, RoutedEventArgs e)
    {
        if (App.MainWindow.ExtendsContentIntoTitleBar)
        {
            // Set the initial interactive regions.
            SetRegionsForCustomTitleBar();
        }
    }

    private void AppTitleBar_SizeChanged(object sender, SizeChangedEventArgs e)
    {
        if (App.MainWindow.ExtendsContentIntoTitleBar)
        {
            // Update interactive regions if the size of the window changes.
            SetRegionsForCustomTitleBar();
        }
    }

    private void TitleBarSearchBox_SizeChanged(object sender, SizeChangedEventArgs e)
    {
        if (App.MainWindow.ExtendsContentIntoTitleBar)
        {
            // Update interactive regions if the size of the window changes.
            SetRegionsForCustomTitleBar();
        }
    }

    private void SetRegionsForCustomTitleBar()
    {
        double scaleAdjustment = AppTitleBar.XamlRoot.RasterizationScale;

        GeneralTransform transform = TitleBarSearchBox.TransformToVisual(null);

        Rect bounds = transform.TransformBounds(new Rect(0, 0, TitleBarSearchBox.ActualWidth, TitleBarSearchBox.ActualHeight));

        RectInt32 SearchBoxRect = GetRect(bounds, scaleAdjustment);

        RectInt32[] rectArray = new[] { SearchBoxRect };

        InputNonClientPointerSource nonClientInputSrc = InputNonClientPointerSource.GetForWindowId(App.MainWindow.AppWindow.Id);
        nonClientInputSrc.SetRegionRects(NonClientRegionKind.Passthrough, rectArray);
    }

    private RectInt32 GetRect(Rect bounds, double scale)
    {
        return new RectInt32(
            _X: (int)Math.Round(bounds.X * scale),
            _Y: (int)Math.Round(bounds.Y * scale),
            _Width: (int)Math.Round(bounds.Width * scale),
            _Height: (int)Math.Round(bounds.Height * scale)
        );
    }

    private void OnLoaded(object sender, RoutedEventArgs e)
    {
        TitleBarHelper.UpdateTitleBar(RequestedTheme);

        KeyboardAccelerators.Add(BuildKeyboardAccelerator(VirtualKey.Left, VirtualKeyModifiers.Menu));
        KeyboardAccelerators.Add(BuildKeyboardAccelerator(VirtualKey.GoBack));

        //MainWindow MW = (MainWindow)App.MainWindow;
        //MW.SetBackdrop(MainWindow.BackdropType.None);
    }

    private void MainWindow_Activated(object sender, WindowActivatedEventArgs args)
    {
        App.AppTitlebar = AppTitleBarText as UIElement;
    }

    private static KeyboardAccelerator BuildKeyboardAccelerator(VirtualKey key, VirtualKeyModifiers? modifiers = null)
    {
        KeyboardAccelerator keyboardAccelerator = new() { Key = key };

        if (modifiers.HasValue)
        {
            keyboardAccelerator.Modifiers = modifiers.Value;
        }

        keyboardAccelerator.Invoked += OnKeyboardAcceleratorInvoked;

        return keyboardAccelerator;
    }

    private static void OnKeyboardAcceleratorInvoked(KeyboardAccelerator sender, KeyboardAcceleratorInvokedEventArgs args)
    {
        INavigationService navigationService = App.GetService<INavigationService>();

        bool result = navigationService.GoBack();

        args.Handled = result;
    }

    private void Quotes_Tapped(object sender, TappedRoutedEventArgs e)
    {
        if (!string.IsNullOrEmpty(Quotes.Text))
        {
            DataPackage Package = new();

            Package.SetText(Quotes.Text);

            Clipboard.SetContent(Package);
        }
    }

    private void Quotes_PointerEntered(object sender, PointerRoutedEventArgs e)
    {
        ProtectedCursor = InputSystemCursor.Create(InputSystemCursorShape.Hand);
    }

    private void Quotes_PointerExited(object sender, PointerRoutedEventArgs e)
    {
        ProtectedCursor = InputSystemCursor.Create(InputSystemCursorShape.Arrow);
    }
}