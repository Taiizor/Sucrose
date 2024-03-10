// Based on:

// hardcodet.net NotifyIcon for WPF
// Copyright (c) 2009 - 2013 Philipp Sumi
// Contact and Information: http://www.hardcodet.net
//
// This library is free software; you can redistribute it and/or
// modify it under the terms of the Code Project Open License (CPOL);
// either version 1.0 of the License, or (at your option) any later
// version.
// 
// The above copyright notice and this permission notice shall be
// included in all copies or substantial portions of the Software.
// 
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND,
// EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES
// OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND
// NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT
// HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY,
// WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING
// FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR
// OTHER DEALINGS IN THE SOFTWARE.
//
// THIS COPYRIGHT NOTICE MAY NOT BE REMOVED FROM THIS FILE

// ReSharper disable InconsistentNaming

namespace WinUIEx.Messaging
{
    // Good list of message numbers: https://wiki.winehq.org/List_Of_Windows_Messages

    /// <summary>
    /// This enum defines the windows messages we respond to.
    /// See more on Windows messages <a href="https://docs.microsoft.com/en-us/windows/win32/learnwin32/window-messages">here</a>
    /// </summary>
    internal enum WindowsMessages : uint
    {
        /// <summary>
        /// Sent after a window has been moved.
        /// See <a href="https://docs.microsoft.com/en-us/windows/win32/winmsg/wm-move">WM_MOVE message</a>.
        /// </summary>
        WM_MOVE = 3,

        /// <summary>
        /// Sent as a signal that a window or an application should terminate.
        /// </summary>
        /// <seealso href="https://docs.microsoft.com/en-us/windows/win32/winmsg/wm-close">Microsoft Docs</seealso>
        WM_CLOSE = 0x0010,

        /// <summary>
        /// A window receives this message when the user chooses a command from the Window menu (formerly known as the
        /// system or control menu) or when the user chooses the maximize button, minimize button, restore button, or 
        /// close button.
        /// </summary>
        /// <seealso href="https://docs.microsoft.com/en-us/windows/win32/menurc/wm-syscommand">Microsoft Docs</seealso>
        WM_SYSCOMMAND = 0x0112,

        /// <summary>
        /// Sent to a window if the mouse causes the cursor to move within a window and mouse input is not captured.
        /// </summary>
        /// <seealso href="https://docs.microsoft.com/en-us/windows/win32/menurc/wm-setcursor">Microsoft Docs</seealso>
        WM_SETCURSOR = 0x20,

        /// <summary>
        /// Posted to a window when the cursor is moved within the nonclient area of the window. This message is posted
        /// to the window that contains the cursor. If a window has captured the mouse, this message is not posted.
        /// </summary>
        /// <seealso href="https://docs.microsoft.com/en-us/windows/win32/inputdev/wm-ncmousemove">Microsoft Docs</seealso>
        WM_NCMOUSEMOVE = 0x00a0,

        /// <summary>
        /// Sent to both the window being activated and the window being deactivated. If the windows use the same input queue,
        /// the message is sent synchronously, first to the window procedure of the top-level window being deactivated, then
        /// to the window procedure of the top-level window being activated. If the windows use different input queues, the
        /// message is sent asynchronously, so the window is activated immediately.
        /// </summary>
        /// <seealso href="https://docs.microsoft.com/en-us/windows/win32/inputdev/wm-activate">Microsoft Docs</seealso>
        WM_ACTIVATE = 0x0006,

        /// <summary>
        /// Sent when a window belonging to a different application than the active window is about to be activated. The
        /// message is sent to the application whose window is being activated and to the application whose window is 
        /// being deactivated.
        /// </summary>
        /// <seealso href="https://docs.microsoft.com/en-us/windows/win32/winmsg/wm-activateapp">Microsoft Docs</seealso>
        WM_ACTIVATEAPP = 0x001c,

        /// <summary>
        /// Sent to a window when the window is about to be hidden or shown.
        /// </summary>
        /// <seealso href="https://docs.microsoft.com/en-us/windows/win32/winmsg/wm-showwindow"/>
        WM_SHOWWINDOW = 0x018,

        /// <summary>
        /// Sent to a window whose size, position, or place in the Z order is about to change as a result of a call to
        /// the SetWindowPos function or another window-management function.
        /// </summary>
        /// <seealso href="https://docs.microsoft.com/en-us/windows/win32/winmsg/wm-windowposchanging">Microsoft Docs</seealso>
        WM_WINDOWPOSCHANGING = 0x0046,

        /// <summary>
        /// Sent to a window whose size, position, or place in the Z order has changed as a result of a call to the SetWindowPos function or another window-management function.
        /// </summary>
        /// <seealso href="https://docs.microsoft.com/en-us/windows/win32/winmsg/wm-windowposchanged">Microsoft Docs</seealso>
        WM_WINDOWPOSCHANGED = 0x0047,

        /// <summary>
        /// Sets the text of a window.
        /// </summary>
        /// <seealso href="https://docs.microsoft.com/en-us/windows/win32/winmsg/wm-settext">Microsoft Docs</seealso>
        WM_SETTEXT = 0x000c,

        /// <summary>
        /// Copies the text that corresponds to a window into a buffer provided by the caller.
        /// </summary>
        /// <seealso href="https://docs.microsoft.com/en-us/windows/win32/winmsg/wm-gettext">Microsoft Docs</seealso>
        WM_GETTEXT = 0x000d,

        /// <summary>
        /// Determines the length, in characters, of the text associated with a window.
        /// </summary>
        /// <seealso href="https://docs.microsoft.com/en-us/windows/win32/winmsg/wm-gettextlength">Microsoft Docs</seealso>
        WM_GETTEXTLENGTH = 0x000e,

        /// <summary>
        /// Sent to a window when its nonclient area needs to be changed to indicate an active or inactive state.
        /// </summary>
        /// <seealso href="https://docs.microsoft.com/en-us/windows/win32/winmsg/wm-ncactivate">Microsoft Docs</seealso>
        WM_NCACTIVATE = 0x0086,

        /// <summary>
        /// Sent to the window that is losing the mouse capture.
        /// </summary>
        /// <seealso href="https://docs.microsoft.com/en-us/windows/win32/inputdev/wm-capturechanged">Microsoft Docs</seealso>
        WM_CAPTURECHANGED = 0x0215,

        /// <summary>
        /// Posted to a window when the cursor leaves the nonclient area of the window specified in a prior call to TrackMouseEvent.
        /// </summary>
        /// <seealso href="https://docs.microsoft.com/en-us/windows/win32/inputdev/wm-ncmouseleave">Microsoft Docs</seealso>
        WM_NCMOUSELEAVE = 0x02a2,

        /// <summary>
        /// Sent after a window has been moved.
        /// See <a href="https://docs.microsoft.com/en-us/windows/win32/winmsg/wm-moving">WM_MOVING message</a>.
        /// </summary>
        WM_MOVING = 0x0216,

        /// <summary>
        /// Sent to a window when a pointer leaves detection range over the window (hover) or when a pointer moves outside the boundaries of the window.
        /// See <a href="https://learn.microsoft.com/en-us/windows/win32/inputmsg/wm-pointerleave">WM_POINTERLEAVE message</a>.
        /// </summary>
        WM_POINTERLEAVE = 0x024A,

        /// <summary>
        /// Posted to provide an update on a pointer that made contact over the client area of a window or on a hovering uncaptured pointer over
        /// the client area of a window. While the pointer is hovering, the message targets whichever window the pointer happens to be over. While
        /// the pointer is in contact with the surface, the pointer is implicitly captured to the window over which the pointer made contact and
        /// that window continues to receive input for the pointer until it breaks contact.
        /// See <a href="https://learn.microsoft.com/en-us/windows/win32/inputmsg/wm-pointerupdate">WM_POINTERUPDATE message</a>.
        /// </summary>
        WM_POINTERUPDATE = 0x0245,

        /// <summary>
        /// Posted to provide an update on a pointer that made contact over the non-client area of a window or when a hovering uncaptured
        /// contact moves over the non-client area of a window. While the pointer is hovering, the message targets whichever window the
        /// pointer happens to be over. While the pointer is in contact with the surface, the pointer is implicitly captured to the window
        /// over which the pointer made contact and that window continues to receive input for the pointer until it breaks contact.
        /// See <a href="https://learn.microsoft.com/en-us/windows/win32/inputmsg/wm-ncpointerupdate">WM_NCPOINTERUPDATE message</a>.
        /// </summary>
        WM_NCPOINTERUPDATE = 0x0241,

        /// <summary>
        /// Sent to a window after its size has changed.
        /// See <a href="https://docs.microsoft.com/en-us/windows/win32/winmsg/wm-size">WM_SIZE message</a>.
        /// </summary>
        WM_SIZE = 0x0005,

        /// <summary>
        /// The WM_NCUAHDRAWCAPTION message is an undocumented message related to themes. When handling WM_NCPAINT, this message should also be handled.
        /// </summary>
        WM_NCUAHDRAWCAPTION = 0x00AE,

        /// <summary>
        /// Sent to a window in order to determine what part of the window corresponds to a particular screen coordinate.
        /// See <a href="https://learn.microsoft.com/en-us/windows/win32/inputdev/wm-nchittest">WM_NCHITTEST message</a>.
        /// </summary>
        WM_NCHITTEST = 0x0084,

        /// <summary>
        /// Sent to a window that the user is resizing. By processing this message, an application can monitor the size and position of the drag rectangle and, if needed, change its size or position.
        /// See <a href="https://docs.microsoft.com/en-us/windows/win32/winmsg/wm-sizing">WM_SIZING message</a>.
        /// </summary>
        WM_SIZING = 0x0214,

        /// <summary>
        /// Sent to a window when the size or position of the window is about to change. An application can use this message to override the window's default maximized size and position, or its default minimum or maximum tracking size.
        /// See <a href="https://docs.microsoft.com/en-us/windows/win32/winmsg/wm-getminmaxinfo">WM_GETMINMAXINFO message</a>.
        /// </summary>
        WM_GETMINMAXINFO = 0x0024,

        /// <summary>
        /// Sent when an application changes the enabled state of a window. It is sent to the window whose enabled state is changing. This message is sent before the <a href="https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-enablewindow">EnableWindow</a> function returns, but after the enabled state (WS_DISABLED style bit) of the window has changed.
        /// See <a href="https://docs.microsoft.com/en-us/windows/win32/winmsg/wm-enable">WM_ENABLE message</a>.
        /// </summary>
        WM_ENABLE = 0x000A,

        /// <summary>
        /// Sent one time to a window after it enters the moving or sizing modal loop. The window enters the moving or sizing modal loop when the user clicks the window's title bar or sizing border, or when the window passes the WM_SYSCOMMAND message to the DefWindowProc function and the wParam parameter of the message specifies the SC_MOVE or SC_SIZE value. 
        /// See <a href="https://docs.microsoft.com/en-us/windows/win32/winmsg/wm-entersizemove">WM_ENTERSIZEMOVE message</a>.
        /// </summary>
        WM_ENTERSIZEMOVE = 0x0231,

        /// <summary>
        /// ent one time to a window, after it has exited the moving or sizing modal loop. The window enters the moving or sizing modal loop when the user clicks the window's title bar or sizing border, or when the window passes the WM_SYSCOMMAND message to the DefWindowProc function and the wParam parameter of the message specifies the SC_MOVE or SC_SIZE value.
        /// See <a href="https://docs.microsoft.com/en-us/windows/win32/winmsg/wm-exitsizemove">WM_EXITSIZEMOVE message</a>.
        /// </summary>
        WM_EXITSIZEMOVE = 0x0232,

        /// <summary>
        /// Notifies a window that the user clicked the right mouse button (right-clicked) in the window.
        /// See <a href="https://docs.microsoft.com/en-us/windows/win32/menurc/wm-contextmenu">WM_CONTEXTMENU message</a>
        /// 
        /// In case of a notify icon: 
        /// If a user selects a notify icon's shortcut menu with the keyboard, the Shell now sends the associated application a WM_CONTEXTMENU message. Earlier versions send WM_RBUTTONDOWN and WM_RBUTTONUP messages.
        /// See <a href="https://docs.microsoft.com/en-us/windows/win32/api/shellapi/nf-shellapi-shell_notifyiconw">Shell_NotifyIcon function</a>
        /// </summary>
        WM_CONTEXTMENU = 0x007b,

        /// <summary>
        /// Posted to a window when the cursor moves.
        /// If the mouse is not captured, the message is posted to the window that contains the cursor.
        /// Otherwise, the message is posted to the window that has captured the mouse.
        ///
        /// See <a href="https://docs.microsoft.com/en-us/windows/win32/inputdev/wm-mousemove">WM_MOUSEMOVE message</a>
        /// </summary>
        WM_MOUSEMOVE = 0x0200,

        /// <summary>
        /// Posted when the user presses the left mouse button while the cursor is in the client area of a window.
        /// If the mouse is not captured, the message is posted to the window beneath the cursor.
        /// Otherwise, the message is posted to the window that has captured the mouse.
        /// 
        /// See <a href="https://docs.microsoft.com/en-us/windows/win32/inputdev/wm-lbuttondown">WM_LBUTTONDOWN message</a>
        /// </summary>
        WM_LBUTTONDOWN = 0x0201,

        /// <summary>
        /// Posted when the user releases the left mouse button while the cursor is in the client area of a window.
        /// If the mouse is not captured, the message is posted to the window beneath the cursor.
        /// Otherwise, the message is posted to the window that has captured the mouse.
        ///
        /// See <a href="https://docs.microsoft.com/en-us/windows/win32/inputdev/wm-lbuttonup">WM_LBUTTONUP message</a>
        /// </summary>
        WM_LBUTTONUP = 0x0202,

        /// <summary>
        /// Posted when the user double-clicks the left mouse button while the cursor is in the client area of a window.
        /// If the mouse is not captured, the message is posted to the window beneath the cursor.
        /// Otherwise, the message is posted to the window that has captured the mouse.
        ///
        /// See <a href="https://docs.microsoft.com/en-us/windows/win32/inputdev/wm-lbuttondblclk">WM_LBUTTONDBLCLK message</a>
        /// </summary>
        WM_LBUTTONDBLCLK = 0x0203,

        /// <summary>
        /// Posted when the user presses the right mouse button while the cursor is in the client area of a window.
        /// If the mouse is not captured, the message is posted to the window beneath the cursor.
        /// Otherwise, the message is posted to the window that has captured the mouse.
        ///
        /// See <a href="https://docs.microsoft.com/en-us/windows/win32/inputdev/wm-rbuttondown">WM_RBUTTONDOWN message</a>
        /// </summary>
        WM_RBUTTONDOWN = 0x0204,

        /// <summary>
        /// Posted when the user releases the right mouse button while the cursor is in the client area of a window.
        /// If the mouse is not captured, the message is posted to the window beneath the cursor.
        /// Otherwise, the message is posted to the window that has captured the mouse.
        ///
        /// See <a href="https://docs.microsoft.com/en-us/windows/win32/inputdev/wm-rbuttonup">WM_RBUTTONUP message</a>
        /// </summary>
        WM_RBUTTONUP = 0x0205,

        /// <summary>
        /// Posted when the user double-clicks the right mouse button while the cursor is in the client area of a window.
        /// If the mouse is not captured, the message is posted to the window beneath the cursor.
        /// Otherwise, the message is posted to the window that has captured the mouse.
        ///
        /// See <a href="https://docs.microsoft.com/en-us/windows/win32/inputdev/wm-rbuttondblclk">WM_RBUTTONDBLCLK message</a>
        /// </summary>
        WM_RBUTTONDBLCLK = 0x0206,

        /// <summary>
        /// Posted when the user presses the middle mouse button while the cursor is in the client area of a window.
        /// If the mouse is not captured, the message is posted to the window beneath the cursor.
        /// Otherwise, the message is posted to the window that has captured the mouse.
        ///
        /// See <a href="https://docs.microsoft.com/en-us/windows/win32/inputdev/wm-mbuttondown">WM_MBUTTONDOWN message</a>
        /// </summary>
        WM_MBUTTONDOWN = 0x0207,

        /// <summary>
        /// Posted when the user releases the middle mouse button while the cursor is in the client area of a window.
        /// If the mouse is not captured, the message is posted to the window beneath the cursor.
        /// Otherwise, the message is posted to the window that has captured the mouse.
        ///
        /// See <a href="https://docs.microsoft.com/en-us/windows/win32/inputdev/wm-mbuttonup">WM_MBUTTONUP message</a>
        /// </summary>
        WM_MBUTTONUP = 0x0208,

        /// <summary>
        /// Posted when the user double-clicks the middle mouse button while the cursor is in the client area of a window.
        /// If the mouse is not captured, the message is posted to the window beneath the cursor.
        /// Otherwise, the message is posted to the window that has captured the mouse.
        ///
        /// See <a href="https://docs.microsoft.com/en-us/windows/win32/inputdev/wm-mbuttondblclk">WM_MBUTTONDBLCLK message</a>
        /// </summary>
        WM_MBUTTONDBLCLK = 0x0209,

        /// <summary>
        /// Used to define private messages for use by private window classes, usually of the form WM_USER+x, where x is an integer value.
        /// </summary>
        WM_USER = 0x0400,

        /// <summary>
        /// Sent to a window to retrieve a handle to the large or small icon associated with a window. The system displays the large icon
        /// in the ALT+TAB dialog, and the small icon in the window caption.
        /// See <a href="https://learn.microsoft.com/en-us/windows/win32/winmsg/wm-geticon">WM_GETICON message</a>
        /// </summary>
        WM_GETICON = 0x007f,

        /// <summary>
        /// Associates a new large or small icon with a window. The system displays the large icon in the ALT+TAB dialog box, and the small icon in the window caption.
        /// </summary>
        WM_SETICON = 0x0080,

        /// <summary>
        /// Sent when the effective dots per inch (dpi) for a window has changed. 
        /// See <a href="https://docs.microsoft.com/en-us/windows/win32/hidpi/wm-dpichanged">WM_DPICHANGED message</a>.
        /// </summary>
        WM_DPICHANGED = 0x02E0,

        /// <summary>
        /// The WM_DISPLAYCHANGE message is sent to all windows when the display resolution has changed.
        /// See <a href="https://docs.microsoft.com/en-us/windows/win32/gdi/wm-displaychange">WM_DISPLAYCHANGE message</a>.
        /// </summary>
        WM_DISPLAYCHANGE = 0x007E,

        /// <summary>
        /// A message that is sent to all top-level windows when the SystemParametersInfo function changes a system-wide setting or when policy settings have changed.
        /// <a href="https://docs.microsoft.com/en-us/windows/win32/winmsg/wm-settingchange">WM_SETTINGCHANGE message</a>
        /// </summary>
        WM_SETTINGCHANGE = 0x001A,

        /// <summary>
        /// Broadcast to every window following a theme change event. Examples of theme change events are the activation of a theme, 
        /// the deactivation of a theme, or a transition from one theme to another.
        /// See <a href="https://docs.microsoft.com/en-us/windows/win32/winmsg/wm-themechanged">WM_THEMECHANGE message</a>
        /// </summary>
        WM_THEMECHANGE = 0x031A,

        /// <summary>
        /// Typically, an application draws in a window in response to a WM_PAINT message. The system sends this message to a window 
        /// procedure when changes to the window have altered the content of the client area. The system sends the message only if 
        /// there are no other messages in the application message queue.
        /// See <a href="https://learn.microsoft.com/en-us/windows/win32/gdi/the-wm-paint-message">WM_PAINT message</a>
        /// </summary>
        WM_PAINT = 0x000F,

        /// <summary>
        /// Sent when the window background must be erased (for example, when a window is resized). The message is sent to prepare 
        /// an invalidated portion of a window for painting.
        /// See <a href="https://learn.microsoft.com/en-us/windows/win32/winmsg/wm-erasebkgnd">WM_ERASEBKGND message</a>
        /// </summary>
        WM_ERASEBKGND = 0x0014,

        /// <summary>
        /// Sent when the size and position of a window's client area must be calculated. By processing this message, an application
        /// can control the content of the window's client area when the size or position of the window changes.
        /// See <a href="https://learn.microsoft.com/en-us/windows/win32/winmsg/wm-nccalcsize">WM_NCCALCSIZE message</a>
        /// </summary>
        WM_NCCALCSIZE = 0x0083,

        /// <summary>
        /// The <see cref="WM_NCPAINT"/> message is sent to a window when its frame must be painted.
        /// See <a href="https://learn.microsoft.com/en-us/windows/win32/gdi/wm-ncpaint">WM_NCPAINT message</a>
        /// </summary>
        WM_NCPAINT = 0x0085,

        /// <summary>
        /// Posted when the user presses the left mouse button while the cursor is within the nonclient area of a window. This message
        /// is posted to the window that contains the cursor. If a window has captured the mouse, this message is not posted.
        /// See <a href="https://learn.microsoft.com/en-us/windows/win32/inputdev/wm-nclbuttondown">WM_NCLBUTTONDOWN message</a>
        /// </summary>
        WM_NCLBUTTONDOWN = 0x00A1,

        /// <summary>
        /// Posted when a pointer makes contact over the non-client area of a window. The message targets the window over which
        /// the pointer makes contact. The pointer is implicitly captured to the window so that the window continues to receive input 
        /// for the pointer until it breaks contact.
        /// See <a href="https://learn.microsoft.com/en-us/windows/win32/inputmsg/wm-ncpointerdown">WM_NCPOINTERDOWN message</a>
        /// </summary>
        WM_NCPOINTERDOWN = 0x0242,

        /// <summary>
        /// Posted when a pointer that made contact over the non-client area of a window breaks contact. The message targets the 
        /// window over which the pointer makes contact and the pointer is, at that point, implicitly captured to the window so 
        /// that the window continues to receive input for the pointer until it breaks contact, including the <see cref="WM_NCPOINTERUP"/>
        /// notification.
        /// See <a href="https://learn.microsoft.com/en-us/windows/win32/inputmsg/wm-ncpointerup">WM_NCPOINTERUP message</a>
        /// </summary>
        WM_NCPOINTERUP = 0x0243,

        /// <summary>
        /// This message is only sent when using NOTIFYICON_VERSION_4, the Shell now sends the associated application an NIN_SELECT notification.
        /// Send when a notify icon is activated with mouse or ENTER key.
        /// Earlier versions send WM_RBUTTONDOWN and WM_RBUTTONUP messages.
        /// </summary>
        NIN_SELECT = WM_USER,

        /// <summary>
        /// This message is only sent when using NOTIFYICON_VERSION_4, the Shell now sends the associated application an NIN_SELECT notification.
        /// Send when a notify icon is activated with SPACEBAR or ENTER key.
        /// Earlier versions send WM_RBUTTONDOWN and WM_RBUTTONUP messages.
        /// </summary>
        NIN_KEYSELECT = WM_USER + 1,

        /// <summary>
        /// Sent when the balloon is shown (balloons are queued).
        /// </summary>
        NIN_BALLOONSHOW = WM_USER + 2,

        /// <summary>
        /// Sent when the balloon disappears. For example, when the icon is deleted.
        /// This message is not sent if the balloon is dismissed because of a timeout or if the user clicks the mouse.
        ///
        /// As of Windows 7, NIN_BALLOONHIDE is also sent when a notification with the NIIF_RESPECT_QUIET_TIME flag set attempts to display during quiet time (a user's first hour on a new computer).
        /// In that case, the balloon is never displayed at all.
        /// </summary>
        NIN_BALLOONHIDE = WM_USER + 3,

        /// <summary>
        /// Sent when the balloon is dismissed because of a timeout.
        /// </summary>
        NIN_BALLOONTIMEOUT = WM_USER + 4,

        /// <summary>
        /// Sent when the balloon is dismissed because the user clicked the mouse.
        /// </summary>
        NIN_BALLOONUSERCLICK = WM_USER + 5,

        /// <summary>
        /// Sent when the user hovers the cursor over an icon to indicate that the richer pop-up UI should be used in place of a standard textual tooltip.
        /// </summary>
        NIN_POPUPOPEN = WM_USER + 6,

        /// <summary>
        /// Sent when a cursor no longer hovers over an icon to indicate that the rich pop-up UI should be closed.
        /// </summary>
        NIN_POPUPCLOSE = WM_USER + 7,
    }
}
