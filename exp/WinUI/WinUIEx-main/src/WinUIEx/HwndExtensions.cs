using System;
using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;
using Windows.Win32;
using Windows.Win32.Foundation;
using Windows.Win32.UI.WindowsAndMessaging;
using Windows.Win32.Graphics.Gdi;
using WinRT;
using Microsoft.UI;

namespace WinUIEx
{
    /// <summary>
    /// A set of HWND Helper Methods
    /// </summary>
    public static class HwndExtensions
    {
        /// <summary>Returns the dots per inch (dpi) value for the associated window.</summary>
        /// <param name = "hwnd">The window you want to get information about.</param>
        /// <returns>The DPI for the window which depends on the <a href = "/windows/desktop/api/windef/ne-windef-dpi_awareness">DPI_AWARENESS</a> of the window. See the Remarks for more information. An invalid <i>hwnd</i> value will result in a return value of 0.</returns>
        /// <remarks>
        /// <para><see href = "https://docs.microsoft.com/windows/win32/api//winuser/nf-winuser-getdpiforwindow">Learn more about this API from docs.microsoft.com</see>.</para>
        /// </remarks>
        public static uint GetDpiForWindow(IntPtr hwnd) => PInvoke.GetDpiForWindow(new HWND(hwnd));

        /// <summary>
        /// Gets the DPI for the monitor that the Window is on
        /// </summary>
        /// <param name="hwnd"></param>
        /// <returns></returns>
        public static uint GetDpiForWindowsMonitor(IntPtr hwnd)
        {
            var hwndDesktop = new HWND(PInvoke.MonitorFromWindow(new(hwnd), MONITOR_FROM_FLAGS.MONITOR_DEFAULTTONEAREST));
            return PInvoke.GetDpiForWindow(new HWND(hwndDesktop));
        }

        /// <summary>Brings the thread that created the specified window into the foreground and activates the window.</summary>
        /// <param name="hWnd">
        /// <para>A handle to the window that should be activated and brought to the foreground.</para>
        /// </param>
        /// <returns>
        /// <para><c>true</c> if the window was brought to the foreground.</para>
        /// <para><c>false</c> if the window was not brought to the foreground.</para>
        /// </returns>
        /// <remarks>
        /// <para><see href = "https://docs.microsoft.com/windows/win32/api//winuser/nf-winuser-setforegroundwindow">Learn more about this API from docs.microsoft.com</see>.</para>
        /// </remarks>
        public static bool SetForegroundWindow(IntPtr hWnd) => PInvoke.SetForegroundWindow(new HWND(hWnd));

        /// <summary>Retrieves the window handle to the active window attached to the calling thread's message queue.</summary>
        /// <returns>
        /// <para>The return value is the handle to the active window attached to the calling thread's message queue. Otherwise, the return value is <b>IntPtr.Zero</b>.</para>
        /// </returns>
        /// <remarks>
        /// <para><see href = "https://docs.microsoft.com/windows/win32/api//winuser/nf-winuser-getactivewindow">Learn more about this API from docs.microsoft.com</see>.</para>
        /// </remarks>
        public static IntPtr GetActiveWindow()
        {
            return (IntPtr)PInvoke.GetActiveWindow().Value;
        }

        /// <summary>Retrieves a handle to the desktop window. The desktop window covers the entire screen. The desktop window is the area on top of which other windows are painted.</summary>
        /// <returns>
        /// <para>The return value is a handle to the desktop window.</para>
        /// </returns>
        /// <remarks>
        /// <para><see href = "https://docs.microsoft.com/windows/win32/api//winuser/nf-winuser-getdesktopwindow">Learn more about this API from docs.microsoft.com</see>.</para>
        /// </remarks>
        public static IntPtr GetDesktopWindow()
        {
            return (IntPtr)PInvoke.GetDesktopWindow().Value;
        }

        /// <summary>
        /// Configures whether the window should always be displayed on top of other windows or not
        /// </summary>
        /// <param name="hwnd">Window handle</param>
        /// <param name="enable">Whether to display on top</param>
        public static void SetAlwaysOnTop(IntPtr hwnd, bool enable)
            => SetWindowPosOrThrow(new HWND(hwnd), new HWND(enable ? -1 : -2), 0, 0, 0, 0, SET_WINDOW_POS_FLAGS.SWP_NOSIZE | SET_WINDOW_POS_FLAGS.SWP_NOMOVE);

        private static void SetWindowPosOrThrow(HWND hWnd, HWND hWndInsertAfter, int X, int Y, int cx, int cy, SET_WINDOW_POS_FLAGS uFlags)
        {
            bool result = PInvoke.SetWindowPos(hWnd, hWndInsertAfter, X, Y, cx, cy, uFlags);
            if (!result)
                Marshal.ThrowExceptionForHR(Marshal.GetLastWin32Error());
        }
        /*public static IntPtr UpdateLayeredWindow(IntPtr hwnd)
        {
            var h = new HWND((nint)hwnd);
            PInvoke.UpdateLayeredWindow(h, )
        }*/

        /// <summary>
        /// Centers the window on the current monitor
        /// </summary>
        /// <param name="hwnd">Window handle</param>
        /// <param name="width">Width of the window in device independent pixels, or <c>null</c> if keeping the current size</param>
        /// <param name="height">Height of the window in device independent pixels, or <c>null</c> if keeping the current size</param>
        public static void CenterOnScreen(IntPtr hwnd, double? width = null, double? height = null)
        {
            var hwndDesktop = PInvoke.MonitorFromWindow(new(hwnd), MONITOR_FROM_FLAGS.MONITOR_DEFAULTTONEAREST);
            MONITORINFO info = new MONITORINFO();
            info.cbSize = 40;
            PInvoke.GetMonitorInfo(hwndDesktop, ref info);
            var dpi = PInvoke.GetDpiForWindow(new HWND(hwnd));
            PInvoke.GetWindowRect(new HWND(hwnd), out RECT windowRect);
            var scalingFactor = dpi / 96d;
            var w = width.HasValue ? (int)(width * scalingFactor) : windowRect.right - windowRect.left;
            var h = height.HasValue ? (int)(height * scalingFactor) : windowRect.bottom - windowRect.top;
            var cx = (info.rcMonitor.left + info.rcMonitor.right) / 2;
            var cy = (info.rcMonitor.bottom + info.rcMonitor.top) / 2;
            var left = cx - (w / 2);
            var top = cy - (h / 2);
            SetWindowPosOrThrow(new HWND(hwnd), new HWND(), left, top, w, h, (SET_WINDOW_POS_FLAGS)0);
        }

        /// <summary>
        /// Sets the icon for the window, using the specified icon ID.
        /// </summary>
        /// <param name="hwnd">Window handle</param>
        /// <param name="iconId">The ID of the icon.</param>
        public static void SetIcon(IntPtr hwnd, IconId iconId)
        {
            var appWindow = WindowExtensions.GetAppWindowFromWindowHandle(hwnd);
            appWindow.SetIcon(iconId);
        }

        /// <summary>
        /// Sets the icon for the window, using the specified icon path.
        /// </summary>
        /// <param name="hwnd">Window handle</param>
        /// <param name="iconPath">The path of the icon.</param>
        public static void SetIcon(IntPtr hwnd, string iconPath)
        {
            var appWindow = WindowExtensions.GetAppWindowFromWindowHandle(hwnd);
            appWindow.SetIcon(iconPath);
        }

        /// <summary>
        /// Positions and resizes the window
        /// </summary>
        /// <param name="hwnd">Window handle</param>
        /// <param name="x">Left side of the window in device independent pixels</param>
        /// <param name="y">Top  side of the window in device independent pixels</param>
        /// <param name="width">Width of the window in device independent pixels, or <c>null</c> if keeping the current size</param>
        /// <param name="height">Height of the window in device independent pixels, or <c>null</c> if keeping the current size</param>
        public static void SetWindowPositionAndSize(IntPtr hwnd, double x, double y, double width, double height)
        {
            var dpi = GetDpiForWindow(hwnd);
            var scalingFactor = dpi / 96d;
            SetWindowPosOrThrow(new HWND(hwnd), new HWND(0), (int)(x * scalingFactor), (int)(y * scalingFactor), (int)(width * scalingFactor), (int)(height * scalingFactor), SET_WINDOW_POS_FLAGS.SWP_NOSENDCHANGING);
        }

        /// <summary>
        /// Sets the width and height of the window in device-independent pixels
        /// </summary>
        /// <param name="hwnd"></param>
        /// <param name="width"></param>
        /// <param name="height"></param>
        public static void SetWindowSize(IntPtr hwnd, double width, double height)
        {
            var dpi = GetDpiForWindow(hwnd);
            var scalingFactor = dpi / 96d;
            SetWindowPosOrThrow(new HWND(hwnd), new HWND(0), 0, 0, (int)(width * scalingFactor), (int)(height * scalingFactor),
                SET_WINDOW_POS_FLAGS.SWP_NOREPOSITION | SET_WINDOW_POS_FLAGS.SWP_NOSENDCHANGING);
        }

        /// <summary>
        /// Sets the task bar icon to the provided icon
        /// </summary>
        /// <param name="hWnd">Window handle</param>
        /// <param name="icon">Icon</param>
        public static void SetTaskBarIcon(IntPtr hWnd, Icon? icon)
        {
            PInvoke.SendMessage(new HWND(hWnd), (uint)Messaging.WindowsMessages.WM_SETICON, new WPARAM(0), new LPARAM(icon?.Handle.Value ?? IntPtr.Zero));
        }

        /// <summary>
        /// Activates the window and displays it in its current size and position.
        /// </summary>
        /// <param name="hWnd">Window handle</param>
        /// <returns><c>true</c> if the window was previously visible, or <c>false</c> if the window was previously hidden.</returns>
        public static bool ShowWindow(IntPtr hWnd) => PInvoke.ShowWindow(new HWND(hWnd), SHOW_WINDOW_CMD.SW_SHOW);

        /// <summary>
        /// Hides the window and activates another window.
        /// </summary>
        /// <param name="hWnd">Window handle</param>
        /// <returns><c>true</c> if the window was previously visible, or <c>false</c> if the window was previously hidden.</returns>
        public static bool HideWindow(IntPtr hWnd) => PInvoke.ShowWindow(new HWND(hWnd), 0);

        /// <summary>
        /// Maximizes the specified window.
        /// </summary>
        /// <param name="hWnd">Window handle</param>
        /// <returns><c>true</c> if the window was previously visible, or <c>false</c> if the window was previously hidden.</returns>
        public static bool MaximizeWindow(IntPtr hWnd) => PInvoke.ShowWindow(new HWND(hWnd), SHOW_WINDOW_CMD.SW_SHOWMAXIMIZED);

        /// <summary>
        /// Minimizes the specified window and activates the next top-level window in the Z order.
        /// </summary>
        /// <param name="hWnd">Window handle</param>
        /// <returns><c>true</c> if the window was previously visible, or <c>false</c> if the window was previously hidden.</returns>
        public static bool MinimizeWindow(IntPtr hWnd) => PInvoke.ShowWindow(new HWND(hWnd), SHOW_WINDOW_CMD.SW_MINIMIZE);

        /// <summary>
        /// Activates and displays the window. If the window is minimized or maximized, the system restores
        /// it to its original size and position. An application should specify this flag when restoring a minimized window.
        /// </summary>
        /// <param name="hWnd">Window handle</param>
        /// <returns><c>true</c> if the window was previously visible, or <c>false</c> if the window was previously hidden.</returns>
        public static bool RestoreWindow(IntPtr hWnd) => PInvoke.ShowWindow(new HWND(hWnd), SHOW_WINDOW_CMD.SW_RESTORE);

        /// <summary>
        /// Gets the current window style
        /// </summary>
        /// <param name="hWnd"></param>
        /// <returns></returns>
        public static WindowStyle GetWindowStyle(IntPtr hWnd)
        {
            var h = new HWND(hWnd);
            return (WindowStyle)PInvoke.GetWindowLong(h, WINDOW_LONG_PTR_INDEX.GWL_STYLE);
        }

        /// <summary>
        /// Disables or enables the window style
        /// </summary>
        /// <param name="hWnd"></param>
        /// <param name="visible"></param>
        /// <param name="style"></param>
        public static void ToggleWindowStyle(IntPtr hWnd, bool visible, WindowStyle style)
        {
            var h = new HWND(hWnd);
            var currentStyle = PInvoke.GetWindowLong(h, WINDOW_LONG_PTR_INDEX.GWL_STYLE);
            var newStyle = currentStyle;
            if (visible)
                newStyle = (newStyle & (int)style);
            else
                newStyle = (newStyle & ~(int)style);
            var r = PInvoke.SetWindowLong(h, WINDOW_LONG_PTR_INDEX.GWL_STYLE, newStyle);
            if (r != currentStyle)
                Marshal.ThrowExceptionForHR(Marshal.GetLastWin32Error());
            // Redraw window
            PInvoke.SetWindowPos(h, new HWND(0), 0, 0, 0, 0, SET_WINDOW_POS_FLAGS.SWP_FRAMECHANGED | SET_WINDOW_POS_FLAGS.SWP_NOMOVE | SET_WINDOW_POS_FLAGS.SWP_NOSIZE | SET_WINDOW_POS_FLAGS.SWP_NOZORDER | SET_WINDOW_POS_FLAGS.SWP_NOOWNERZORDER);
        }

        /// <summary>
        /// Sets the current window style
        /// </summary>
        /// <param name="hWnd">Window handle</param>
        /// <param name="newStyle"></param>
        public static void SetWindowStyle(IntPtr hWnd, WindowStyle newStyle)
        {
            var h = new HWND(hWnd);
            var currentStyle = PInvoke.GetWindowLong(h, WINDOW_LONG_PTR_INDEX.GWL_STYLE);
            var r = PInvoke.SetWindowLong(h, WINDOW_LONG_PTR_INDEX.GWL_STYLE, (int)newStyle);
            if (r != currentStyle)
                Marshal.ThrowExceptionForHR(Marshal.GetLastWin32Error());
            // Redraw window
            PInvoke.SetWindowPos(h, new HWND(0), 0, 0, 0, 0, SET_WINDOW_POS_FLAGS.SWP_FRAMECHANGED | SET_WINDOW_POS_FLAGS.SWP_NOMOVE | SET_WINDOW_POS_FLAGS.SWP_NOSIZE | SET_WINDOW_POS_FLAGS.SWP_NOZORDER | SET_WINDOW_POS_FLAGS.SWP_NOOWNERZORDER);
        }
        /// <summary>
        /// Gets the current window style
        /// </summary>
        /// <param name="hWnd"></param>
        /// <returns></returns>
        public static ExtendedWindowStyle GetExtendedWindowStyle(IntPtr hWnd)
        {
            var h = new HWND(hWnd);
            return (ExtendedWindowStyle)PInvoke.GetWindowLong(h, WINDOW_LONG_PTR_INDEX.GWL_EXSTYLE);
        }

        /// <summary>
        /// Sets the current window style
        /// </summary>
        /// <param name="hWnd">Window handle</param>
        /// <param name="newStyle"></param>
        public static void SetExtendedWindowStyle(IntPtr hWnd, ExtendedWindowStyle newStyle)
        {
            var h = new HWND(hWnd);
            var currentStyle = PInvoke.GetWindowLong(h, WINDOW_LONG_PTR_INDEX.GWL_EXSTYLE);
            var r = PInvoke.SetWindowLong(h, WINDOW_LONG_PTR_INDEX.GWL_EXSTYLE, (int)newStyle);
            if (r != currentStyle) // If the function succeeds, the return value is the previous value of the specified 32-bit integer.
                Marshal.ThrowExceptionForHR(Marshal.GetLastWin32Error());
            // Redraw window
            PInvoke.SetWindowPos(h, new HWND(0), 0, 0, 0, 0, SET_WINDOW_POS_FLAGS.SWP_FRAMECHANGED | SET_WINDOW_POS_FLAGS.SWP_NOMOVE | SET_WINDOW_POS_FLAGS.SWP_NOSIZE | SET_WINDOW_POS_FLAGS.SWP_NOZORDER | SET_WINDOW_POS_FLAGS.SWP_NOOWNERZORDER);
        }

        /// <summary>
        /// Disables or enables the window style
        /// </summary>
        /// <param name="hWnd"></param>
        /// <param name="visible"></param>
        /// <param name="style"></param>
        public static void ToggleExtendedWindowStyle(IntPtr hWnd, bool visible, ExtendedWindowStyle style)
        {
            var h = new HWND(hWnd);
            var currentStyle = PInvoke.GetWindowLong(h, WINDOW_LONG_PTR_INDEX.GWL_EXSTYLE);
            var newStyle = currentStyle;
            if (visible)
                newStyle = (newStyle & (int)style);
            else
                newStyle = (newStyle & ~(int)style);
            var r = PInvoke.SetWindowLong(h, WINDOW_LONG_PTR_INDEX.GWL_EXSTYLE, newStyle);
            if (r != currentStyle) // If the function succeeds, the return value is the previous value of the specified 32-bit integer.
                Marshal.ThrowExceptionForHR(Marshal.GetLastWin32Error());
            // Redraw window
            PInvoke.SetWindowPos(h, new HWND(0), 0, 0, 0, 0, SET_WINDOW_POS_FLAGS.SWP_FRAMECHANGED | SET_WINDOW_POS_FLAGS.SWP_NOMOVE | SET_WINDOW_POS_FLAGS.SWP_NOSIZE | SET_WINDOW_POS_FLAGS.SWP_NOZORDER | SET_WINDOW_POS_FLAGS.SWP_NOOWNERZORDER);
        }

        /// <summary>
        /// Sets the opacity of a layered window.
        /// </summary>
        /// <param name="hWnd">Window handle</param>
        /// <param name="alpha">Alpha value used to describe the opacity of the layered window. When <paramref name="alpha"/> is 0, the window is completely transparent. When <paramref name="alpha"/> is 255, the window is opaque.</param>
        //// <seealso cref="SetLayeredWindowAttributes(IntPtr, byte, byte, byte, byte)"/>
        public static void SetWindowOpacity(IntPtr hWnd, byte alpha)
        {
            var handle = new HWND(hWnd);
            var extendedStyle = (WindowStyles)PInvoke.GetWindowLong(handle, WINDOW_LONG_PTR_INDEX.GWL_EXSTYLE);
            if(extendedStyle.HasFlag(WindowStyles.WS_EX_LAYERED) == false)
                PInvoke.SetWindowLong(handle, WINDOW_LONG_PTR_INDEX.GWL_EXSTYLE, (int) (extendedStyle | WindowStyles.WS_EX_LAYERED));
            if (!PInvoke.SetLayeredWindowAttributes(handle, 0, alpha, LAYERED_WINDOW_ATTRIBUTES_FLAGS.LWA_ALPHA))
                Marshal.ThrowExceptionForHR(Marshal.GetLastWin32Error());
        }
        [Flags]
        private enum WindowStyles : int
        {
            WS_EX_LAYERED = 0x00080000,
        }

        /* Chroma keys doesn't seem to work on WinUI windows
        /// <summary>
        /// Sets the opacity of a layered window.
        /// </summary>
        /// <param name="hWnd">Window handle</param>
        /// <param name="chromaKeyRed">The red part of the color that specifies the transparency color key to be used when composing the layered window. All pixels painted by the window in this color will be transparent.</param>
        /// <param name="chromaKeyGreen">The green part of the color that specifies the transparency color key to be used when composing the layered window. All pixels painted by the window in this color will be transparent.</param>
        /// <param name="chromaKeyBlue">The blue part of the color that specifies the transparency color key to be used when composing the layered window. All pixels painted by the window in this color will be transparent.</param>
        /// <seealso cref="SetLayeredWindowAttributes(IntPtr, byte, byte, byte, byte)"/>
        public static void SetWindowChromaKey(IntPtr hWnd, byte chromaKeyRed, byte chromaKeyGreen, byte chromaKeyBlue)
        {
            var handle = new HWND(hWnd);
            var extendedStyle = (WindowStyles)PInvoke.GetWindowLong(handle, WINDOW_LONG_PTR_INDEX.GWL_EXSTYLE);
            if (extendedStyle.HasFlag(WindowStyles.WS_EX_LAYERED) == false)
                PInvoke.SetWindowLong(handle, WINDOW_LONG_PTR_INDEX.GWL_EXSTYLE, (int)(extendedStyle | WindowStyles.WS_EX_LAYERED));
            uint color = (uint)((chromaKeyRed) | (chromaKeyGreen << 8) | chromaKeyBlue << 16);
            if (!PInvoke.SetLayeredWindowAttributes(handle , color, 0, LAYERED_WINDOW_ATTRIBUTES_FLAGS.LWA_COLORKEY))
                Marshal.ThrowExceptionForHR(Marshal.GetLastWin32Error());
        }

        /// <summary>
        /// Sets the opacity and transparency color key of a layered window.
        /// </summary>
        /// <param name="hWnd">Window handle</param>
        /// <param name="chromaKeyRed">The red part of the color that specifies the transparency color key to be used when composing the layered window. All pixels painted by the window in this color will be transparent.</param>
        /// <param name="chromaKeyGreen">The green part of the color that specifies the transparency color key to be used when composing the layered window. All pixels painted by the window in this color will be transparent.</param>
        /// <param name="chromaKeyBlue">The blue part of the color that specifies the transparency color key to be used when composing the layered window. All pixels painted by the window in this color will be transparent.</param>
        /// <param name="alpha">Alpha value used to describe the opacity of the layered window. When <paramref name="alpha"/> is 0, the window is completely transparent. When <paramref name="alpha"/> is 255, the window is opaque.</param>
        public static void SetLayeredWindowAttributes(IntPtr hWnd, byte chromaKeyRed, byte chromaKeyGreen, byte chromaKeyBlue, byte alpha)
        {
            var handle = new HWND(hWnd);
            var extendedStyle = (WindowStyles)PInvoke.GetWindowLong(handle, WINDOW_LONG_PTR_INDEX.GWL_EXSTYLE);
            if (extendedStyle.HasFlag(WindowStyles.WS_EX_LAYERED) == false)
                PInvoke.SetWindowLong(handle, WINDOW_LONG_PTR_INDEX.GWL_EXSTYLE, (int)(extendedStyle | WindowStyles.WS_EX_LAYERED));

            uint color = (uint)((chromaKeyRed) | (chromaKeyGreen << 8) | chromaKeyBlue << 16);
            if (!PInvoke.SetLayeredWindowAttributes(handle, color, alpha, LAYERED_WINDOW_ATTRIBUTES_FLAGS.LWA_COLORKEY | LAYERED_WINDOW_ATTRIBUTES_FLAGS.LWA_ALPHA))
                Marshal.ThrowExceptionForHR(Marshal.GetLastWin32Error());
        }*/
    }

    /// <summary>
    /// Flags used for ToggleWindowStyle method
    /// </summary>
    [Flags]
    public enum WindowStyle : int
    {
        /// <summary>The window has a thin-line border.</summary>
        Border = 0x00800000,
        /// <summary>The window has a title bar (includes the BORDER style).</summary>
        Caption = 0x00C00000,
        /// <summary>The window is a child window. A window with this style cannot have a menu bar. This style cannot be used with the POPUP style.</summary>
        Child = 0x40000000,
        /// <summary>Same as the <see cref="Child"/> style.</summary>
        ChildWindow = 0x40000000,
        /// <summary>Excludes the area occupied by child windows when drawing occurs within the parent window. This style is used when creating the parent window.</summary>
        ChildChildren = 0x02000000,
        /// <summary>Clips child windows relative to each other; that is, when a particular child window receives a WM_PAINT message, the CLIPSIBLINGS style clips all other overlapping child windows out of the region of the child window to be updated. If CLIPSIBLINGS is not specified and child windows overlap, it is possible, when drawing within the client area of a child window, to draw within the client area of a neighboring child window.</summary>
        ClipSiblings = 0x04000000,
        /// <summary>The window is initially disabled. A disabled window cannot receive input from the user. To change this after a window has been created, use the EnableWindow function.</summary>
        Disabled = 0x08000000,
        /// <summary>The window has a border of a style typically used with dialog boxes. A window with this style cannot have a title bar.</summary>
        DlgFrame = 0x00400000,
        /// <summary>The window is the first control of a group of controls. The group consists of this first control and all controls defined after it, up to the next control with the GROUP style. The first control in each group usually has the TABSTOP style so that the user can move from group to group. The user can subsequently change the keyboard focus from one control in the group to the next control in the group by using the direction keys.
        /// You can turn this style on and off to change dialog box navigation. To change this style after a window has been created, use the SetWindowLong function.</summary>
        Group = 0x00020000,
        /// <summary>The window has a horizontal scroll bar.</summary>
        HScroll = 0x00100000,
        /// <summary>The window is initially minimized. Same as the MINIMIZE style.</summary>
        Iconic = 0x20000000,
        /// <summary>The window is initially maximized.</summary>
        Maximize = 0x01000000,
        /// <summary>The window has a maximize button. Cannot be combined with the EX_CONTEXTHELP style. The SYSMENU style must also be specified.</summary>
        MaximizeBox = 0x00010000,
        /// <summary>The window is initially minimized. Same as the ICONIC style.</summary>
        Minimize = 0x20000000,
        /// <summary>The window has a minimize button. Cannot be combined with the EX_CONTEXTHELP style. The SYSMENU style must also be specified.</summary>
        MinimizeBox = 0x00020000,
        /// <summary>The window is an overlapped window. An overlapped window has a title bar and a border. Same as the TILED style.</summary>
        Overlapped = 0x00000000,
        /// <summary>The window is an overlapped window. Same as the TILEDWINDOW style.</summary>
        OverlappedWindow = (Overlapped | Caption | SysMenu | ThickFrame | MinimizeBox | MaximizeBox),
        // <summary>The window is a pop-up window. This style cannot be used with the CHILD style.</summary>
        //POPUP = 0x80000000,
        // <summary>The window is a pop-up window. The CAPTION and POPUPWINDOW styles must be combined to make the window menu visible.</summary>
        //POPUPWINDOW = (POPUP | BORDER | SYSMENU),
        /// <summary>The window has a sizing border. Same as the THICKFRAME style.</summary>
        SizeBox = 0x00040000,
        /// <summary>The window has a window menu on its title bar. The CAPTION style must also be specified.</summary>
        SysMenu = 0x00080000,
        /// <summary>The window is a control that can receive the keyboard focus when the user presses the TAB key. Pressing the TAB key changes the keyboard focus to the next control with the TABSTOP style.
        /// You can turn this style on and off to change dialog box navigation. To change this style after a window has been created, use the SetWindowLong function. For user-created windows and modeless dialogs to work with tab stops, alter the message loop to call the IsDialogMessage function.</summary>
        TabStop = 0x00010000,
        /// <summary>The window has a sizing border. Same as the SIZEBOX style.</summary>
        ThickFrame = 0x00040000,
        /// <summary>The window is an overlapped window. An overlapped window has a title bar and a border. Same as the OVERLAPPED style.</summary>
        Tiled = 0x00000000,
        /// <summary>The window is an overlapped window. Same as the OVERLAPPEDWINDOW style.</summary>
        TiledWindow = (Overlapped | Caption | SysMenu | ThickFrame | MinimizeBox | MaximizeBox),
        /// <summary>The window is initially visible.
        /// This style can be turned on and off by using the ShowWindow or SetWindowPos function.</summary>
        Visible = 0x10000000,
        /// <summary>The window has a vertical scroll bar.</summary>
        VScroll = 0x00200000,
    } 

    /// <summary>
    /// Flags used for ToggleExtendedWindowStyle method
    /// </summary>
    [Flags]
    public enum ExtendedWindowStyle : int
    {
        /// <summary>The window accepts drag-drop files.</summary>
        AcceptFiles = 0x00000010,
        /// <summary>Forces a top-level window onto the taskbar when the window is visible.</summary>
        AppWindow = 0x00040000,
        /// <summary>The window has a border with a sunken edge.</summary>
        ClientEdge = 0x00000200,
        /// <summary>Paints all descendants of a window in bottom-to-top painting order using double-buffering. Bottom-to-top painting order allows a descendent window to have translucency (alpha) and transparency (color-key) effects, but only if the descendent window also has the WS_EX_TRANSPARENT bit set. Double-buffering allows the window and its descendents to be painted without flicker. This cannot be used if the window has a class style of either CS_OWNDC or CS_CLASSDC.
        /// Windows 2000: This style is not supported.</summary>
        Composited = 0x02000000,
        /// <summary>The title bar of the window includes a question mark. When the user clicks the question mark, the cursor changes to a question mark with a pointer. If the user then clicks a child window, the child receives a WM_HELP message. The child window should pass the message to the parent window procedure, which should call the WinHelp function using the HELP_WM_HELP command. The Help application displays a pop-up window that typically contains help for the child window.
        /// WS_EX_CONTEXTHELP cannot be used with the WS_MAXIMIZEBOX or WS_MINIMIZEBOX styles.</summary>
        ContextHelp = 0x00000400,
        /// <summary>The window itself contains child windows that should take part in dialog box navigation. If this style is specified, the dialog manager recurses into children of this window when performing navigation operations such as handling the TAB key, an arrow key, or a keyboard mnemonic.</summary>
        ControlParent = 0x00010000,
        /// <summary>The window has a double border; the window can, optionally, be created with a title bar by specifying the WS_CAPTION style in the dwStyle parameter.</summary>
        DlgModalFrame = 0x00000001,
        /// <summary> The window is a layered window. This style cannot be used if the window has a class style of either CS_OWNDC or CS_CLASSDC.
        /// Windows 8: The WS_EX_LAYERED style is supported for top-level windows and child windows. Previous Windows versions support WS_EX_LAYERED only for top-level windows.</summary>
        Layered = 0x00080000,
        /// <summary> If the shell language is Hebrew, Arabic, or another language that supports reading order alignment, the horizontal origin of the window is on the right edge. Increasing horizontal values advance to the left.</summary>
        LayoutRtl = 0x00400000,
        /// <summary>The window has generic left-aligned properties. This is the default.</summary>
        Left = 0x00000000,
        /// <summary>If the shell language is Hebrew, Arabic, or another language that supports reading order alignment, the vertical scroll bar (if present) is to the left of the client area. For other languages, the style is ignored.</summary>
        LeftScrollBar = 0x00004000,
        /// <summary>The window text is displayed using left-to-right reading-order properties. This is the default.</summary>
        LtrReading = 0x00000000,
        /// <summary>The window is a MDI child window.</summary>
        MdiChild = 0x00000040,
        /// <summary>A top-level window created with this style does not become the foreground window when the user clicks it. The system does not bring this window to the foreground when the user minimizes or closes the foreground window.
        /// The window should not be activated through programmatic access or via keyboard navigation by accessible technology, such as Narrator.
        /// To activate the window, use the SetActiveWindow or SetForegroundWindow function.
        /// The window does not appear on the taskbar by default. To force the window to appear on the taskbar, use the WS_EX_APPWINDOW style.</summary>
        NoActivate = 0x08000000,
        /// <summary>The window does not pass its window layout to its child windows.</summary>
        NoInheritLayout = 0x00100000,
        /// <summary>The child window created with this style does not send the WM_PARENTNOTIFY message to its parent window when it is created or destroyed.</summary>
        NoParentNotify = 0x00000004,
        /// <summary>The window does not render to a redirection surface. This is for windows that do not have visible content or that use mechanisms other than surfaces to provide their visual.</summary>
        NoRedirectionBitmap = 0x00200000,
        /// <summary>The window is an overlapped window.</summary>
        OverlappedWindow = (WindowEdge | ClientEdge),
        /// <summary>The window is palette window, which is a modeless dialog box that presents an array of commands.</summary>
        PaletteWindow = (WindowEdge | ToolWindow | TopMost),
        /// <summary>The window has generic "right-aligned" properties. This depends on the window class. This style has an effect only if the shell language is Hebrew, Arabic, or another language that supports reading-order alignment; otherwise, the style is ignored.
        /// Using the WS_EX_RIGHT style for static or edit controls has the same effect as using the SS_RIGHT or ES_RIGHT style, respectively. Using this style with button controls has the same effect as using BS_RIGHT and BS_RIGHTBUTTON styles.</summary>
        Right = 0x00001000,
        /// <summary>The vertical scroll bar (if present) is to the right of the client area. This is the default.</summary>
        RightScrollBar = 0x00000000,
        /// <summary>If the shell language is Hebrew, Arabic, or another language that supports reading-order alignment, the window text is displayed using right-to-left reading-order properties. For other languages, the style is ignored.</summary>
        RtlReading = 0x00002000,
        /// <summary>The window has a three-dimensional border style intended to be used for items that do not accept user input.</summary>
        StaticEdge = 0x00020000,
        /// <summary>The window is intended to be used as a floating toolbar. A tool window has a title bar that is shorter than a normal title bar, and the window title is drawn using a smaller font. A tool window does not appear in the taskbar or in the dialog that appears when the user presses ALT+TAB. If a tool window has a system menu, its icon is not displayed on the title bar. However, you can display the system menu by right-clicking or by typing ALT+SPACE.</summary>
        ToolWindow = 0x00000080,
        /// <summary>The window should be placed above all non-topmost windows and should stay above them, even when the window is deactivated. To add or remove this style, use the SetWindowPos function.</summary>
        TopMost = 0x00000008,
        /// <summary>The window should not be painted until siblings beneath the window (that were created by the same thread) have been painted. The window appears transparent because the bits of underlying sibling windows have already been painted.
        /// To achieve transparency without these restrictions, use the SetWindowRgn function.</summary>
        Transparent = 0x00000020,
        /// <summary>The window has a border with a raised edge.</summary>
        WindowEdge = 0x00000100,
    }
}
