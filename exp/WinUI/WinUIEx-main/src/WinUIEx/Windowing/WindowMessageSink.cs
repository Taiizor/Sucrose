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


using System;
using System.Collections.Concurrent;
using System.ComponentModel;
using System.Diagnostics;
using Windows.Win32;
using Windows.Win32.Foundation;
using Windows.Win32.UI.WindowsAndMessaging;
using Windows.Win32.Graphics.Gdi;
using WinUIEx.Messaging;

namespace WinUIEx
{
    /// <summary>
    /// Receives messages from the taskbar icon through
    /// window messages of an underlying helper window.
    /// </summary>
    internal unsafe class WindowMessageSink : IDisposable
    {
        #region members

        /// <summary>
        /// The ID of messages that are received from the the
        /// taskbar icon.
        /// </summary>
        public const int CallbackMessageId = 0x400;

        /// <summary>
        /// The ID of the message that is being received if the
        /// taskbar is (re)started.
        /// </summary>
        private uint taskbarRestartMessageId;

        /// <summary>
        /// Used to track whether a mouse-up event is just
        /// the aftermath of a double-click and therefore needs
        /// to be suppressed.
        /// </summary>
        private bool isDoubleClick;

        //internal delegate LRESULT WindowProcedureHandler(HWND hWnd, uint uMsg, WPARAM wParam, LPARAM lParam);

        // <summary>
        // A delegate that processes messages of the hidden
        // native window that receives window messages. Storing
        // this reference makes sure we don't loose our reference
        // to the message window.
        // </summary>
        //private WindowProcedureHandler messageHandler;

        /// <summary>
        /// Window class ID.
        /// </summary>
        internal string? WindowId { get; private set; }

        /// <summary>
        /// Handle for the message window.
        /// </summary> 
        internal HWND MessageWindowHandle { get; private set; }

        #endregion

        #region events

        /// <summary>
        /// The custom tooltip should be closed or hidden.
        /// </summary>
        public event Action<bool>? ChangeToolTipStateRequest;

        /// <summary>
        /// Fired in case the user clicked or moved within
        /// the taskbar icon area.
        /// </summary>
        public event Action<MouseEvent>? MouseEventReceived;

        /// <summary>
        /// Fired if a balloon ToolTip was either displayed
        /// or closed (indicated by the boolean flag).
        /// </summary>
        public event Action<bool>? BalloonToolTipChanged;

        /// <summary>
        /// Fired if the taskbar was created or restarted. Requires the taskbar
        /// icon to be reset.
        /// </summary>
        public event Action? TaskbarCreated;

        #endregion

        #region construction

        /// <summary>
        /// Creates a new message sink that receives message from
        /// a given taskbar icon.
        /// </summary>
        public WindowMessageSink() : this(false)
        {            
        }


        private WindowMessageSink(bool asEmpty)
        {
            if(!asEmpty)
                CreateMessageWindow();
        }

        /// <summary>
        /// Creates a dummy instance that provides an empty
        /// pointer rather than a real window handler.<br/>
        /// Used at design time.
        /// </summary>
        /// <returns>WindowMessageSink</returns>
        internal static WindowMessageSink CreateEmpty()
        {
            return new WindowMessageSink(true)
            {
                MessageWindowHandle = new HWND(0),
            };
        }

        #endregion

        #region CreateMessageWindow

        /// <summary>
        /// Creates the helper message window that is used
        /// to receive messages from the taskbar icon.
        /// </summary>
        private void CreateMessageWindow()
        {
            //generate a unique ID for the window
            WindowId = "TaskbarIcon_" + Guid.NewGuid();

            //register window message handler
            // Create a simple window class which is reference through
            WNDCLASSW wc = new WNDCLASSW();
            wc.style = 0;
            wc.lpfnWndProc = OnWindowMessageReceived;
            wc.cbClsExtra = 0;
            wc.cbWndExtra = 0;
            wc.hInstance = new HINSTANCE(IntPtr.Zero);
            wc.hIcon = new HICON(IntPtr.Zero);
            wc.hCursor = new HCURSOR(IntPtr.Zero);
            wc.hbrBackground = new HBRUSH(IntPtr.Zero);
            fixed (char* id = string.Empty)
                wc.lpszMenuName = id;
            fixed (char* id = WindowId)
                wc.lpszClassName = id;

            // Register the window class
            PInvoke.RegisterClass(wc);

            // Get the message used to indicate the taskbar has been restarted
            // This is used to re-add icons when the taskbar restarts
            taskbarRestartMessageId = PInvoke.RegisterWindowMessage("TaskbarCreated");

            var hwnd = new HWND();
            //var hwnd = PInvoke.GetActiveWindow();
            // Create the message window
            MessageWindowHandle = PInvoke.CreateWindowEx(0, WindowId, "", 0, 0, 0, 1, 1, hwnd, null, null, null);

            if (MessageWindowHandle == IntPtr.Zero)
            {
                throw new Win32Exception("Message window handle was not a valid pointer");
            }
        }
        #endregion CreateMessageWindow

        #region Handle Window Messages

        /// <summary>
        /// Callback method that receives messages from the taskbar area.
        /// </summary>
        private LRESULT OnWindowMessageReceived(HWND hWnd, uint messageId, WPARAM wParam, LPARAM lParam)
        {
            if (messageId == taskbarRestartMessageId)
            {
                //recreate the icon if the taskbar was restarted (e.g. due to Win Explorer shutdown)
                var listener = TaskbarCreated;
                listener?.Invoke();
            }

            //forward message
            ProcessWindowMessage(messageId, wParam, lParam);
            return PInvoke.DefWindowProc(hWnd, messageId, wParam, lParam);
        }

        /// <summary>
        /// Processes incoming system messages.
        /// </summary>
        /// <param name="msg">Callback ID.</param>
        /// <param name="wParam">If the version is Vista
        /// or higher, this parameter can be used to resolve mouse coordinates.
        /// Currently not in use.</param>
        /// <param name="lParam">Provides information about the event.</param>
        private void ProcessWindowMessage(uint msg, WPARAM wParam, LPARAM lParam)
        {
            if (msg != CallbackMessageId) return;

            var message = (WindowsMessages) lParam.Value;
            Debug.WriteLine("Got message " + message);
            switch (message)
            {
                case WindowsMessages.WM_CONTEXTMENU:
                    // TODO: Handle WM_CONTEXTMENU, see https://docs.microsoft.com/en-us/windows/win32/api/shellapi/nf-shellapi-shell_notifyiconw
                    Debug.WriteLine("Unhandled WM_CONTEXTMENU");
                    break;

                case WindowsMessages.WM_MOUSEMOVE:
                    MouseEventReceived?.Invoke(MouseEvent.MouseMove);
                    break;

                case WindowsMessages.WM_LBUTTONDOWN:
                    MouseEventReceived?.Invoke(MouseEvent.IconLeftMouseDown);
                    break;

                case WindowsMessages.WM_LBUTTONUP:
                    if (!isDoubleClick)
                    {
                        MouseEventReceived?.Invoke(MouseEvent.IconLeftMouseUp);
                    }
                    isDoubleClick = false;
                    break;

                case WindowsMessages.WM_LBUTTONDBLCLK:
                    isDoubleClick = true;
                    MouseEventReceived?.Invoke(MouseEvent.IconDoubleClick);
                    break;

                case WindowsMessages.WM_RBUTTONDOWN:
                    MouseEventReceived?.Invoke(MouseEvent.IconRightMouseDown);
                    break;

                case WindowsMessages.WM_RBUTTONUP:
                    MouseEventReceived?.Invoke(MouseEvent.IconRightMouseUp);
                    break;

                case WindowsMessages.WM_RBUTTONDBLCLK:
                    //double click with right mouse button - do not trigger event
                    break;

                case WindowsMessages.WM_MBUTTONDOWN:
                    MouseEventReceived?.Invoke(MouseEvent.IconMiddleMouseDown);
                    break;

                case WindowsMessages.WM_MBUTTONUP:
                    MouseEventReceived?.Invoke(MouseEvent.IconMiddleMouseUp);
                    break;

                case WindowsMessages.WM_MBUTTONDBLCLK:
                    //double click with middle mouse button - do not trigger event
                    break;

                case WindowsMessages.NIN_BALLOONSHOW:
                    BalloonToolTipChanged?.Invoke(true);
                    break;

                case WindowsMessages.NIN_BALLOONHIDE:
                case WindowsMessages.NIN_BALLOONTIMEOUT:
                    BalloonToolTipChanged?.Invoke(false);
                    break;

                case WindowsMessages.NIN_BALLOONUSERCLICK:
                    MouseEventReceived?.Invoke(MouseEvent.BalloonToolTipClicked);
                    break;

                case WindowsMessages.NIN_POPUPOPEN:
                    ChangeToolTipStateRequest?.Invoke(true);
                    break;

                case WindowsMessages.NIN_POPUPCLOSE:
                    ChangeToolTipStateRequest?.Invoke(false);
                    break;

                case WindowsMessages.NIN_SELECT:
                    // TODO: Handle NIN_SELECT see https://docs.microsoft.com/en-us/windows/win32/api/shellapi/nf-shellapi-shell_notifyiconw
                    Debug.WriteLine("Unhandled NIN_SELECT");
                    break;

                case WindowsMessages.NIN_KEYSELECT:
                    // TODO: Handle NIN_KEYSELECT see https://docs.microsoft.com/en-us/windows/win32/api/shellapi/nf-shellapi-shell_notifyiconw
                    Debug.WriteLine("Unhandled NIN_KEYSELECT");
                    break;

                default:
                    Debug.WriteLine("Unhandled NotifyIcon message ID: " + lParam);
                    break;
            }
        }

        #endregion

        #region Dispose

        /// <summary>
        /// Set to true as soon as <c>Dispose</c> has been invoked.
        /// </summary>
        public bool IsDisposed { get; private set; }


        /// <summary>
        /// Disposes the object.
        /// </summary>
        /// <remarks>This method is not virtual by design. Derived classes
        /// should override <see cref="Dispose(bool)"/>.
        /// </remarks>
        public void Dispose()
        {
            Dispose(true);

            // This object will be cleaned up by the Dispose method.
            // Therefore, you should call GC.SuppressFinalize to
            // take this object off the finalization queue 
            // and prevent finalization code for this object
            // from executing a second time.
            GC.SuppressFinalize(this);
        }

        /// <summary>
        /// This destructor will run only if the <see cref="Dispose()"/>
        /// method does not get called. This gives this base class the
        /// opportunity to finalize.
        /// <para>
        /// Important: Do not provide destructor in types derived from
        /// this class.
        /// </para>
        /// </summary>
        ~WindowMessageSink()
        {
            Dispose(false);
        }

        /// <summary>
        /// Removes the windows hook that receives window
        /// messages and closes the underlying helper window.
        /// </summary>
        private void Dispose(bool disposing)
        {
            //don't do anything if the component is already disposed
            if (IsDisposed) return;
            IsDisposed = true;

            //always destroy the unmanaged handle (even if called from the GC)
            PInvoke.DestroyWindow(MessageWindowHandle);
        }

        #endregion
    }
}