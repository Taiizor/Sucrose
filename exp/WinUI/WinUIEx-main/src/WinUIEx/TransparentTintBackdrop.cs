using Microsoft.UI.Composition;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Hosting;
using System;
using System.Runtime.InteropServices;
using Windows.Win32;
using WinRT;
using WinUIEx.Messaging;

namespace WinUIEx
{
    /// <summary>
    /// A custom backdrop that make the window completely transparent.
    /// </summary>
    public class TransparentTintBackdrop : CompositionBrushBackdrop
    {
        private WindowMessageMonitor? monitor;
        private Windows.UI.Composition.CompositionColorBrush? brush;

        /// <summary>
        /// Initializes a new instance of the <see cref="TransparentTintBackdrop"/> class.
        /// </summary>
        public TransparentTintBackdrop() : this(Microsoft.UI.Colors.Transparent) 
        {
        }

        /// <summary>
        /// Initializes a new instance of the <see cref="TransparentTintBackdrop"/> class.
        /// </summary>
        /// <param name="tintColor">Color for the background. The Alpha value defines the opacity of the window</param>
        public TransparentTintBackdrop(Windows.UI.Color tintColor)
        {
            _color = tintColor;
        }

        private Windows.UI.Color _color;

        /// <summary>
        /// Gets or sets the color used for the backdrop. The Alpha value defines the opacity of the window.
        /// </summary>
        public Windows.UI.Color TintColor
        {
            get { return _color; }
            set
            {
                _color = value;
                if (brush != null)
                {
                    brush.Color = value;
                }
            }
        }

        /// <inheritdoc />
        protected override Windows.UI.Composition.CompositionBrush CreateBrush(Windows.UI.Composition.Compositor compositor)
        {
            return brush = WindowManager.Compositor.CreateColorBrush(TintColor);
        }

        /// <inheritdoc />
        protected override void OnTargetConnected(ICompositionSupportsSystemBackdrop connectedTarget, XamlRoot xamlRoot)
        {
            var inspectable = connectedTarget.As<IInspectable>();
            var xamlSource = DesktopWindowXamlSource.FromAbi(inspectable.ThisPtr);
            var hWnd = xamlSource.SiteBridge.SiteView.EnvironmentView.AppWindowId.Value;

            monitor = new WindowMessageMonitor((IntPtr)hWnd);
            monitor.WindowMessageReceived += Monitor_WindowMessageReceived;

            ConfigureDwm(hWnd);

            base.OnTargetConnected(connectedTarget, xamlRoot);

            var hdc = PInvoke.GetDC(new Windows.Win32.Foundation.HWND((nint)hWnd));
            ClearBackground((nint)hWnd, hdc.Value);
        }

        /// <inheritdoc />
        protected override void OnTargetDisconnected(ICompositionSupportsSystemBackdrop disconnectedTarget)
        {
            monitor?.Dispose();
            monitor = null;
            var backdrop = disconnectedTarget.SystemBackdrop;
            disconnectedTarget.SystemBackdrop = null;
            backdrop?.Dispose();
            brush?.Dispose();
            brush = null;
            base.OnTargetDisconnected(disconnectedTarget);
        }

        private static void ConfigureDwm(ulong hWnd)
        {
            Windows.Win32.Foundation.HWND handle = new Windows.Win32.Foundation.HWND((nint)hWnd);
            PInvoke.DwmExtendFrameIntoClientArea(handle, new Windows.Win32.UI.Controls.MARGINS());
            PInvoke.DwmEnableBlurBehindWindow(handle, new Windows.Win32.Graphics.Dwm.DWM_BLURBEHIND()
            {
                dwFlags = 3,
                fEnable = true,
                hRgnBlur = PInvoke.CreateRectRgn(-2, -2, -1, -1),
            });
        }

        private bool ClearBackground(nint hwnd, nint hdc)
        {
            if (PInvoke.GetClientRect(new Windows.Win32.Foundation.HWND(hwnd), out var rect))
            {
                var brush = PInvoke.CreateSolidBrush(0);
                FillRect(hdc, ref rect, brush);
                return true;
            }
            return false;
        }

        private unsafe void Monitor_WindowMessageReceived(object? sender, WindowMessageEventArgs e)
        {
            if (e.MessageType == WindowsMessages.WM_ERASEBKGND)
            {
                if (ClearBackground(e.Message.Hwnd, (nint)e.Message.WParam))
                {
                    e.Result = 1;
                    e.Handled = true;
                }
            }
            else if ((int)e.MessageType == 798 /*WM_DWMCOMPOSITIONCHANGED*/)
            {
                ConfigureDwm((ulong)e.Message.Hwnd);
                e.Handled = true;
                e.Result = 0;
            }
        }

        [DllImport("User32", ExactSpelling = true)]
        [DefaultDllImportSearchPaths(DllImportSearchPath.System32)]
        private static extern unsafe int FillRect(IntPtr hDC, ref Windows.Win32.Foundation.RECT lprc, Windows.Win32.Graphics.Gdi.HBRUSH hbr);
    }
}
