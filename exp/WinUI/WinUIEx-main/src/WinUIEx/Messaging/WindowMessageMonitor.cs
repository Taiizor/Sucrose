using System;

namespace WinUIEx.Messaging
{
    /// <summary>
    /// The message monitor allows you to monitor all WM_MESSAGE events for a given window.
    /// </summary>
    public sealed class WindowMessageMonitor : IDisposable
    {
        private IntPtr _hwnd = IntPtr.Zero;
        private delegate IntPtr WinProc(IntPtr hWnd, uint msg, IntPtr wParam, IntPtr lParam);
        private Windows.Win32.UI.Shell.SUBCLASSPROC? callback;
        private readonly object _lockObject = new object();

        /// <summary>
        /// Initialize a new instance of the <see cref="WindowMessageMonitor"/> class.
        /// </summary>
        /// <param name="window">The window to listen to messages for</param>
        public WindowMessageMonitor(Microsoft.UI.Xaml.Window window) : this(window.GetWindowHandle())
        {
        }

        /// <summary>
        /// Initialize a new instance of the <see cref="WindowMessageMonitor"/> class.
        /// </summary>
        /// <param name="hwnd">The window handle to listen to messages for</param>
        public WindowMessageMonitor(IntPtr hwnd)
        {
            _hwnd = hwnd;
        }

        /// <summary>
        /// Finalizer
        /// </summary>
        ~WindowMessageMonitor()
        {
            Dispose(false);
        }

        /// <summary>
        /// Disposes this instance
        /// </summary>
        public void Dispose() => Dispose(true);

        private void Dispose(bool disposing)
        { 
            if (_NativeMessage != null)
                Unsubscribe();
        }

        private event EventHandler<WindowMessageEventArgs>? _NativeMessage;

        /// <summary>
        /// Event raised when a windows message is received.
        /// </summary>
        public event EventHandler<WindowMessageEventArgs> WindowMessageReceived
        {
            add
            {
                if (_NativeMessage is null)
                {
                    Subscribe();
                }
                _NativeMessage += value;
            }
            remove
            {
                _NativeMessage -= value;
                if (_NativeMessage is null)
                {
                    Unsubscribe();
                }
            }
        }

        private Windows.Win32.Foundation.LRESULT NewWindowProc(Windows.Win32.Foundation.HWND hWnd, uint uMsg, Windows.Win32.Foundation.WPARAM wParam, Windows.Win32.Foundation.LPARAM lParam, nuint uIdSubclass, nuint dwRefData)
        {
            var handler = _NativeMessage;
            if (handler != null)
            {
                var args = new WindowMessageEventArgs(hWnd, uMsg, wParam.Value, lParam);
                handler.Invoke(this, args);
                if (args.Handled)
                    return new Windows.Win32.Foundation.LRESULT((int)args.Result);
            }
            return Windows.Win32.PInvoke.DefSubclassProc(hWnd, uMsg, wParam, lParam);
        }

        private void Subscribe()
        {
            lock (_lockObject)
                if (callback == null)
                {
                    callback = new Windows.Win32.UI.Shell.SUBCLASSPROC(NewWindowProc);
                    bool ok = Windows.Win32.PInvoke.SetWindowSubclass(new Windows.Win32.Foundation.HWND(_hwnd), callback, 101, 0);
                }
        }

        private void Unsubscribe()
        {
            lock (_lockObject)
                if (callback != null)
                {
                    Windows.Win32.PInvoke.RemoveWindowSubclass(new Windows.Win32.Foundation.HWND(_hwnd), callback, 101);
                    callback = null;
                }
        }
    }
}
