using Microsoft.UI;
using Microsoft.UI.Windowing;
using System;
using System.Runtime.InteropServices;

namespace WinUIEx
{
    /// <summary>
    /// WinUI Window Extension Methods
    /// </summary>
    public static partial class WindowExtensions
    {
        /// <summary>
        /// Gets the AppWindow from the handle
        /// </summary>
        /// <param name="window"></param>
        /// <returns></returns>
        [Obsolete("Use Microsoft.UI.Xaml.Window.AppWindow")]
        public static AppWindow GetAppWindow(this Microsoft.UI.Xaml.Window window) => window.AppWindow;

        /// <summary>Returns the dots per inch (dpi) value for the associated window.</summary>
        /// <param name = "window">The window you want to get information about.</param>
        /// <returns>The DPI for the window which depends on the <a href = "/windows/desktop/api/windef/ne-windef-dpi_awareness">DPI_AWARENESS</a> of the window. See the Remarks for more information. An invalid <i>hwnd</i> value will result in a return value of 0.</returns>
        /// <remarks>
        /// <para><see href = "https://docs.microsoft.com/windows/win32/api//winuser/nf-winuser-getdpiforwindow">Learn more about this API from docs.microsoft.com</see>.</para>
        /// </remarks>
        public static uint GetDpiForWindow(this Microsoft.UI.Xaml.Window window) => HwndExtensions.GetDpiForWindow(window.GetWindowHandle());

        /// <summary>Brings the thread that created the specified window into the foreground and activates the window.</summary>
        /// <param name="window">
        /// <para>The window that should be activated and brought to the foreground.</para>
        /// </param>
        /// <returns>
        /// <para><c>true</c> if the window was brought to the foreground.</para>
        /// <para><c>false</c> if the window was not brought to the foreground.</para>
        /// </returns>
        public static bool SetForegroundWindow(this Microsoft.UI.Xaml.Window window) => HwndExtensions.SetForegroundWindow(window.GetWindowHandle());

        /// <summary>
        /// Configures whether the window should always be displayed on top of other windows or not
        /// </summary>
        /// <remarks>The presenter must be an overlapped presenter.</remarks>
        /// <exception cref="NotSupportedException">Throw if the AppWindow Presenter isn't an overlapped presenter.</exception>
        /// <param name="window">Window</param>
        /// <param name="enable">Whether to display on top</param>
        public static void SetIsAlwaysOnTop(this Microsoft.UI.Xaml.Window window, bool enable) => window.UpdateOverlappedPresenter((c) => c.IsAlwaysOnTop = enable);

        /// <summary>
        /// Gets a value indicating whether this window is on top or not.
        /// </summary>
        /// <param name="window">window</param>
        /// <returns><c>True</c> if the overlapped presenter is on top, otherwise <c>false</c>.</returns>
        public static bool GetIsAlwaysOnTop(this Microsoft.UI.Xaml.Window window) => window.GetOverlappedPresenterValue((c) => c?.IsAlwaysOnTop ?? false);

        /// <summary>
        /// Enables or disables the ability to resize the window.
        /// </summary>
        /// <remarks>The presenter must be an overlapped presenter.</remarks>
        /// <exception cref="NotSupportedException">Throw if the AppWindow Presenter isn't an overlapped presenter.</exception>
        /// <param name="window"></param>
        /// <param name="enable"></param>
        public static void SetIsResizable(this Microsoft.UI.Xaml.Window window, bool enable) => window.UpdateOverlappedPresenter((c) => c.IsResizable = enable);

        /// <summary>
        /// Gets a value indicating whether this resizable or not.
        /// </summary>
        /// <param name="window">window</param>
        /// <returns><c>True</c> if the overlapped presenter is resizeable, otherwise <c>false</c>.</returns>
        public static bool GetIsResizable(this Microsoft.UI.Xaml.Window window) => GetOverlappedPresenterValue(window, (c) => c?.IsResizable ?? false);

        /// <summary>
        /// </summary>
        /// <remarks>The presenter must be an overlapped presenter.</remarks>
        /// <exception cref="NotSupportedException">Throw if the AppWindow Presenter isn't an overlapped presenter.</exception>
        /// <param name="window">window</param>
        /// <param name="enable"><c>true</c> if this window should be maximizable.</param>
        public static void SetIsMaximizable(this Microsoft.UI.Xaml.Window window, bool enable) => window.UpdateOverlappedPresenter((c) => c.IsMaximizable = enable);

        /// <summary>
        /// Gets a value indicating whether this window is maximizeable or not.
        /// </summary>
        /// <param name="window">window</param>
        /// <returns><c>True</c> if the overlapped presenter is on maximizable, otherwise <c>false</c>.</returns>
        public static bool GetIsMaximizable(this Microsoft.UI.Xaml.Window window) => GetOverlappedPresenterValue(window, (c) => c?.IsMaximizable ?? false);

        /// <summary>
        /// </summary>
        /// <remarks>The presenter must be an overlapped presenter.</remarks>
        /// <exception cref="NotSupportedException">Throw if the AppWindow Presenter isn't an overlapped presenter.</exception>
        /// <param name="window">window</param>
        /// <param name="enable"><c>true</c> if this window should be minimizable.</param>
        public static void SetIsMinimizable(this Microsoft.UI.Xaml.Window window, bool enable) => window.UpdateOverlappedPresenter((c) => c.IsMinimizable = enable);

        /// <summary>
        /// Gets a value indicating whether this window is minimizeable or not.
        /// </summary>
        /// <param name="window">window</param>
        /// <returns><c>True</c> if the overlapped presenter is on minimizable, otherwise <c>false</c>.</returns>
        public static bool GetIsMinimizable(this Microsoft.UI.Xaml.Window window) => GetOverlappedPresenterValue(window, (c) => c?.IsMinimizable ?? false);

        /// <summary>
        /// Enables or disables showing the window in the task switchers.
        /// </summary>
        /// <param name="window">window</param>
        /// <param name="enable"><c>true</c> if this window should be shown in the task switchers, otherwise <c>false</c>.</param>
        public static void SetIsShownInSwitchers(this Microsoft.UI.Xaml.Window window, bool enable) => window.AppWindow.IsShownInSwitchers = enable;

        /// <summary>
        /// Sets the icon for the window, using the specified icon ID.
        /// </summary>
        /// <param name="window">Window</param>
        /// <param name="iconId">The ID of the icon.</param>
        public static void SetIcon(this Microsoft.UI.Xaml.Window window, IconId iconId) => HwndExtensions.SetIcon(window.GetWindowHandle(), iconId);

        private static void UpdateOverlappedPresenter(this Microsoft.UI.Xaml.Window window, Action<OverlappedPresenter> action)
        {
            if (window is null)
                throw new ArgumentNullException(nameof(window));
            var appwindow = window.AppWindow;
            if (appwindow.Presenter is OverlappedPresenter overlapped)
                action(overlapped);
            else 
                throw new NotSupportedException($"Not supported with a {appwindow.Presenter.Kind} presenter");
        }
        private static T GetOverlappedPresenterValue<T>(this Microsoft.UI.Xaml.Window window, Func<OverlappedPresenter?,T> action)
        {
            if (window is null)
                throw new ArgumentNullException(nameof(window));
            var appwindow = window.AppWindow;
            return action(appwindow.Presenter as OverlappedPresenter);
        }

        /// <summary>
        /// Centers the window on the current monitor
        /// </summary>
        /// <param name="window">Window</param>
        /// <param name="width">Width of the window in device independent pixels, or <c>null</c> if keeping the current size</param>
        /// <param name="height">Height of the window in device independent pixels, or <c>null</c> if keeping the current size</param>
        public static void CenterOnScreen(this Microsoft.UI.Xaml.Window window, double? width = null, double? height = null)
            => HwndExtensions.CenterOnScreen(window.GetWindowHandle(), width, height);

        /// <summary>
        /// Positions and resizes the window
        /// </summary>
        /// <param name="window">Window</param>
        /// <param name="x">Left side of the window</param>
        /// <param name="y">Top side of the window</param>
        /// <param name="width">Width of the window in device independent pixels.</param>
        /// <param name="height">Height of the window in device independent pixels.</param>
        public static void MoveAndResize(this Microsoft.UI.Xaml.Window window, double x, double y, double width, double height)
        {
            var scale = HwndExtensions.GetDpiForWindow(window.GetWindowHandle()) / 96f;
            window.AppWindow.MoveAndResize(new Windows.Graphics.RectInt32((int)x, (int)y, (int)(width * scale), (int)(height * scale)));
        }

        /// <summary>
        /// Positions the window
        /// </summary>
        /// <param name="window">Window</param>
        /// <param name="x">Left side of the window</param>
        /// <param name="y">Top side of the window</param>
        public static void Move(this Microsoft.UI.Xaml.Window window, int x, int y)
        {
            window.AppWindow.MoveAndResize(new Windows.Graphics.RectInt32(x, y, window.AppWindow.Size.Width, window.AppWindow.Size.Height));
        }

        /// <summary>
        /// Sets the width and height of the window in device-independent pixels.
        /// </summary>
        /// <param name="window">Window to set the size for.</param>
        /// <param name="width">Width of the window in device-independent units.</param>
        /// <param name="height">Height of the window in device-independent units.</param>
        public static void SetWindowSize(this Microsoft.UI.Xaml.Window window, double width, double height)
        {
            var scale = HwndExtensions.GetDpiForWindow(window.GetWindowHandle()) / 96f;
            window.AppWindow.Resize(new Windows.Graphics.SizeInt32((int)(width * scale), (int)(height * scale)));
        }

        /// <summary>
        /// Sets the window presenter kind used.
        /// </summary>
        /// <param name="window"></param>
        /// <param name="kind"></param>
        /// <returns></returns>
        public static void SetWindowPresenter(this Microsoft.UI.Xaml.Window window, AppWindowPresenterKind kind) => window.AppWindow.SetPresenter(kind);

        /// <summary>
        /// Gets the native HWND pointer handle for the window
        /// </summary>
        /// <param name="window">The window to return the handle for</param>
        /// <returns>HWND handle</returns>
        public static IntPtr GetWindowHandle(this Microsoft.UI.Xaml.Window window)
            => window is null ? throw new ArgumentNullException(nameof(window)) : WinRT.Interop.WindowNative.GetWindowHandle(window);

        [DllImport("Microsoft.Internal.FrameworkUdk.dll", EntryPoint = "Windowing_GetWindowHandleFromWindowId", CharSet = CharSet.Unicode)]
        private static extern IntPtr GetWindowHandleFromWindowId(WindowId windowId, out IntPtr result);

        /// <summary>
        /// Gets the window HWND handle from a Window ID.
        /// </summary>
        /// <param name="windowId">Window ID to get handle from</param>
        /// <returns>Window HWND handle</returns>
        public static IntPtr GetWindowHandle(this WindowId windowId)
        {
            IntPtr hwnd;
            GetWindowHandleFromWindowId(windowId, out hwnd);
            return hwnd;
        }

        /// <summary>
        /// Gets the AppWindow from an HWND
        /// </summary>
        /// <param name="hwnd"></param>
        /// <returns>AppWindow</returns>
        public static AppWindow GetAppWindowFromWindowHandle(IntPtr hwnd)
        {
            if(hwnd == IntPtr.Zero)
                    throw new ArgumentNullException(nameof(hwnd));
            WindowId windowId = Win32Interop.GetWindowIdFromWindow(hwnd);
            return AppWindow.GetFromWindowId(windowId);
        }

        /// <summary>
        /// Activates the window and displays it in its current size and position.
        /// </summary>
        /// <param name="window">Window</param>
        /// <returns><c>true</c> if the window was previously visible, or <c>false</c> if the window was previously hidden.</returns>
        public static bool Show(this Microsoft.UI.Xaml.Window window) => HwndExtensions.ShowWindow(GetWindowHandle(window));

        /// <summary>
        /// Hides the window and activates another window.
        /// </summary>
        /// <param name="window">Window</param>
        /// <returns><c>true</c> if the window was previously visible, or <c>false</c> if the window was previously hidden.</returns>
        public static bool Hide(this Microsoft.UI.Xaml.Window window) => HwndExtensions.HideWindow(GetWindowHandle(window));

        /// <summary>
        /// Maximizes the specified window.
        /// </summary>
        /// <remarks>The presenter must be an overlapped presenter.</remarks>
        /// <exception cref="NotSupportedException">Throw if the AppWindow Presenter isn't an overlapped presenter.</exception>
        /// <param name="window">Window</param>
        public static void Maximize(this Microsoft.UI.Xaml.Window window) => UpdateOverlappedPresenter(window, (c) => c.Maximize());

        /// <summary>
        /// Minimizes the specified window and activates the next top-level window in the Z order.
        /// </summary>
        /// <remarks>The presenter must be an overlapped presenter.</remarks>
        /// <exception cref="NotSupportedException">Throw if the AppWindow Presenter isn't an overlapped presenter.</exception>
        /// <param name="window">Window</param>
        public static void Minimize(this Microsoft.UI.Xaml.Window window) => UpdateOverlappedPresenter(window, (c) => c.Minimize());

        /// <summary>
        /// Activates and displays the window. If the window is minimized or maximized, the system restores
        /// it to its original size and position.
        /// </summary>
        /// <remarks>The presenter must be an overlapped presenter.</remarks>
        /// <exception cref="NotSupportedException">Throw if the AppWindow Presenter isn't an overlapped presenter.</exception>
        /// <param name="window">Window</param>
        public static void Restore(this Microsoft.UI.Xaml.Window window) => UpdateOverlappedPresenter(window, (c) => c.Restore());

        /// <summary>
        /// Sets the icon for the window, using the specified icon path.
        /// </summary>
        /// <param name="window">Window</param>
        /// <param name="iconPath">The path of the icon.</param>
        public static void SetIcon(this Microsoft.UI.Xaml.Window window, string iconPath) => HwndExtensions.SetIcon(window.GetWindowHandle(), iconPath);
        /// <summary>
        /// Sets the task bar icon to the provided icon
        /// </summary>
        /// <param name="window">Window</param>
        /// <param name="icon">Icon</param>
        public static void SetTaskBarIcon(this Microsoft.UI.Xaml.Window window, Icon? icon) => HwndExtensions.SetTaskBarIcon(window.GetWindowHandle(), icon);

        /// <summary>
        /// Gets the background color for the title bar and all its buttons and their states.
        /// </summary>
        /// <param name="window">window</param>
        /// <param name="color">color</param>
        public static void SetTitleBarBackgroundColors(this Microsoft.UI.Xaml.Window window, Windows.UI.Color color)
        {
            var appWindow = window.AppWindow;
            if (AppWindowTitleBar.IsCustomizationSupported())
            {
                appWindow.TitleBar.ButtonBackgroundColor = color;
                appWindow.TitleBar.BackgroundColor = color;
                appWindow.TitleBar.ButtonInactiveBackgroundColor = color;
                appWindow.TitleBar.ButtonPressedBackgroundColor = color;
                appWindow.TitleBar.InactiveBackgroundColor = color;
            }
        }

        /// <summary>
        /// Gets the current window style
        /// </summary>
        /// <param name="window">Window</param>
        /// <returns></returns>
        public static WindowStyle GetWindowStyle(this Microsoft.UI.Xaml.Window window) => HwndExtensions.GetWindowStyle(window.GetWindowHandle());

        /// <summary>
        /// Sets the current window style
        /// </summary>
        /// <param name="window">Window</param>
        /// <param name="newStyle"></param>
        public static void SetWindowStyle(this Microsoft.UI.Xaml.Window window, WindowStyle newStyle) => HwndExtensions.SetWindowStyle(window.GetWindowHandle(), newStyle);

        /// <summary>
        /// Disables or enables the window style
        /// </summary>
        /// <param name="window">Window</param>
        /// <param name="visible"></param>
        /// <param name="style"></param>
        public static void ToggleWindowStyle(this Microsoft.UI.Xaml.Window window, bool visible, WindowStyle style) => HwndExtensions.ToggleWindowStyle(window.GetWindowHandle(), visible, style);

        /// <summary>
        /// Gets the current window style
        /// </summary>
        /// <param name="window">Window</param>
        /// <returns></returns>
        public static ExtendedWindowStyle GetExtendedWindowStyle(this Microsoft.UI.Xaml.Window window) => HwndExtensions.GetExtendedWindowStyle(window.GetWindowHandle());

        /// <summary>
        /// Sets the current window style
        /// </summary>
        /// <param name="window">Window</param>
        /// <param name="newStyle"></param>
        public static void SetExtendedWindowStyle(this Microsoft.UI.Xaml.Window window, ExtendedWindowStyle newStyle) => HwndExtensions.SetExtendedWindowStyle(window.GetWindowHandle(), newStyle);

        /// <summary>
        /// Disables or enables the window style
        /// </summary>
        /// <param name="window">Window</param>
        /// <param name="visible"></param>
        /// <param name="style"></param>
        public static void ToggleExtendedWindowStyle(this Microsoft.UI.Xaml.Window window, bool visible, ExtendedWindowStyle style) => HwndExtensions.ToggleExtendedWindowStyle(window.GetWindowHandle(), visible, style);

        /// <summary>
        /// Sets the opacity of a layered window.
        /// </summary>
        /// <param name="window">window</param>
        /// <param name="alpha">Alpha value used to describe the opacity of the layered window. When <paramref name="alpha"/> is 0, the window is completely transparent. When <paramref name="alpha"/> is 255, the window is opaque.</param>
        //// <seealso cref="SetLayeredWindowAttributes(Microsoft.UI.Xaml.Window, Windows.UI.Color, byte)" />
        public static void SetWindowOpacity(this Microsoft.UI.Xaml.Window window, byte alpha)
            => HwndExtensions.SetWindowOpacity(GetWindowHandle(window), alpha);

        /* Chroma keys doesn't seem to work on WinUI windows
        /// <summary>
        /// Sets the opacity of a layered window.
        /// </summary>
        /// <param name="window">window</param>
        /// <param name="chromaKey">The color that specifies the transparency color key to be used when composing the layered window. All pixels painted by the window in this color will be transparent.</param>
        /// <seealso cref="SetLayeredWindowAttributes(Microsoft.UI.Xaml.Window, Windows.UI.Color, byte)" />
        public static void SetWindowChromaKey(this Microsoft.UI.Xaml.Window window, Windows.UI.Color chromaKey)
            => HwndExtensions.SetWindowChromaKey(GetWindowHandle(window), chromaKey.R, chromaKey.G, chromaKey.B);

        /// <summary>
        /// Sets the opacity and transparency color key of a layered window.
        /// </summary>
        /// <param name="window">window</param>
        /// <param name="chromaKey">The color that specifies the transparency color key to be used when composing the layered window. All pixels painted by the window in this color will be transparent.</param>
        /// <param name="alpha">Alpha value used to describe the opacity of the layered window. When <paramref name="alpha"/> is 0, the window is completely transparent. When <paramref name="alpha"/> is 255, the window is opaque.</param>
        public static void SetLayeredWindowAttributes(this Microsoft.UI.Xaml.Window window, Windows.UI.Color chromaKey, byte alpha) 
            => HwndExtensions.SetLayeredWindowAttributes(GetWindowHandle(window), chromaKey.R, chromaKey.G, chromaKey.B, alpha);*/
    }
}
