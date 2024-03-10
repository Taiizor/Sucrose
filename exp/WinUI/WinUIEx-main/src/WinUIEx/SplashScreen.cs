using System;
using System.ComponentModel;
using System.Threading.Tasks;
using Microsoft.UI.Dispatching;
using Microsoft.UI.Xaml;

namespace WinUIEx
{
    /// <summary>
    /// A splash screen window that shows with no chrome, and once <see cref="SplashScreen.OnLoading"/> has completed,
    /// opens a new window
    /// </summary>
    public class SplashScreen : Window
    {
        private Window? _window;
        private Type? _windowType;
        private readonly WindowManager _manager;

        /// <summary>
        /// Creates and activates a new splashscreen, and opens the specified window once complete.
        /// </summary>
        /// <param name="window">Window to open once splash screen is complete</param>
        public SplashScreen(Window window) : this()
        {
            _window = window ?? throw new ArgumentNullException(nameof(window));
        }

        /// <summary>
        /// Creates and activates a new splashscreen, and creates and opens the specified window type once complete.
        /// </summary>
        /// <param name="window">Type of window to create. Must have an empty constructor</param>
        [System.Diagnostics.CodeAnalysis.RequiresUnreferencedCode("The type's constructor must not be trimmed. Use the Window instance constructor instead to ensure trim safety.")]
        public SplashScreen(Type window) : this()
        {
            if (!window.IsSubclassOf(typeof(Window)) && window != typeof(Window))
                throw new ArgumentException("Type must be a Window");
            _windowType = window ?? throw new ArgumentNullException(nameof(window));
        }

        private SplashScreen()
        {
            this.Activated += SplashScreen_Activated;
            this.DispatcherQueue.TryEnqueue(DispatcherQueuePriority.Normal, () =>
            {
                this.Activate();
            });
            _manager = WindowManager.Get(this);
        }

        /// <summary>
        /// Gets or sets the system backdrop of the window.
        /// Note: Windows 10 doesn't support these, so will fall back to default backdrop .
        /// </summary>
        [Obsolete("Use Microsoft.UI.Xaml.Window.SystemBackdrop")]
        [EditorBrowsable(EditorBrowsableState.Never)]
        public SystemBackdrop? Backdrop
        {
            get => _manager.Backdrop;
            set => _manager.Backdrop = value;
        }

        private async void Content_Loaded(object sender, RoutedEventArgs e)
        {
            if (IsAlwaysOnTop)
                this.SetIsAlwaysOnTop(true);
            else
                WindowExtensions.SetForegroundWindow(this);
            double h = Height;
            double w = Width;
            if (Content is FrameworkElement f)
            {
                if (double.IsNaN(Width) && f.DesiredSize.Width > 0)
                    w = f.DesiredSize.Width;
                if (double.IsNaN(Height) && f.DesiredSize.Height > 0)
                    h = f.DesiredSize.Height;
            }
            if (double.IsNaN(w))
                w = 640;
            if (double.IsNaN(h))
                h = 480;

            this.CenterOnScreen(w, h);
            this.Show();
            await OnLoading();
            if (_windowType != null)
#pragma warning disable IL2077 // Target parameter argument does not satisfy 'DynamicallyAccessedMembersAttribute' in call to target method. Type parameter already declared as requiring unreferenced code
                _window = Activator.CreateInstance(_windowType) as Window;
#pragma warning restore IL2077 // Target parameter argument does not satisfy 'DynamicallyAccessedMembersAttribute' in call to target method. The source field does not have matching annotations.
            _window?.Activate();
            this.Close();
            _window?.SetForegroundWindow();
            Completed?.Invoke(this, _window);
            _window = null;
        }

        private void SplashScreen_Activated(object sender, WindowActivatedEventArgs args)
        {
            this.Activated -= SplashScreen_Activated;
            this.Hide(); // Hides until content is loaded
            var hwnd = this.GetWindowHandle();
            HwndExtensions.ToggleWindowStyle(hwnd, false, WindowStyle.TiledWindow);
            var content = this.Content as FrameworkElement;
            if (content is null || content.IsLoaded)
                Content_Loaded(this, new RoutedEventArgs());
            else
                content.Loaded += Content_Loaded;
        }

        /// <summary>
        /// Override to display loading progress or delay loading of main window
        /// </summary>
        /// <returns></returns>
        protected virtual Task OnLoading() => Task.CompletedTask;

        /// <summary>
        /// Gets or sets the width of the splash screen. Set to NaN to size for content
        /// </summary>
        public double Width { get; set; } = 640;
        /// <summary>
        /// Gets or sets the height of the splash screen. Set to NaN to size for content
        /// </summary>
        public double Height { get; set; } = 480;

        /// <summary>
        /// Gets or sets a value indicating whether the splash screen should be top-most
        /// </summary>
        public bool IsAlwaysOnTop { get; set; } = false;

        /// <summary>
        /// Raised once the splash screen has completed <see cref="OnLoading"/>.
        /// </summary>
        public event EventHandler<Window?>? Completed;
    }
}