using CefSharp;
using Skylark.Struct.Monitor;
using Skylark.Wing.Helper;
using Skylark.Wing.Utility;
using System.Runtime.InteropServices;
using System.Windows;

namespace Sucrose.WPF.CS
{
    /// <summary>
    /// Interaction logic for MainWindow.xaml
    /// </summary>
    public partial class MainWindow : Window
    {
        public int ScreenIndex { get; private set; } = 0;

        public bool IsFixed { get; private set; } = false;

        public bool State { get; private set; } = false;

        private static IntPtr MouseHook = IntPtr.Zero;

        public MainWindow()
        {
            InitializeComponent();

            PinToBackground();

            WallView.MenuHandler = new CustomContextMenuHandler();
        }

        private void WallView_Loaded(object sender, RoutedEventArgs e)
        {
            State = true;

            MouseEventCall = CatchMouseEvent;
            MouseHook = SetWindowsHookEx(14, MouseEventCall, IntPtr.Zero, 0);
        }

        protected bool PinToBackground()
        {
            IsFixed = DesktopIcon.FixWindow(this);

            if (IsFixed)
            {
                Screene.FillScreenWindow(this, OwnerScreen);
            }

            return IsFixed;
        }

        public int OwnerScreenIndex
        {
            get => ScreenIndex;
            set
            {
                if (value < 0)
                {
                    value = 0;
                }
                else if (value >= Screen.AllScreens.Length)
                {
                    value = 0;
                }

                if (ScreenIndex != value)
                {
                    ScreenIndex = value;

                    PinToBackground();
                }
            }
        }

        public MonitorStruct OwnerScreen
        {
            get
            {
                if (OwnerScreenIndex < Screene.Screens.Length)
                {
                    return Screene.Screens[OwnerScreenIndex];
                }

                return new MonitorStruct()
                {
                    rcMonitor = Screen.PrimaryScreen.Bounds,
                    rcWork = Screen.PrimaryScreen.WorkingArea,
                };
            }
        }

        private enum MouseMessages
        {
            WM_LBUTTONDOWN = 0x0201,
            WM_LBUTTONUP = 0x0202,
            WM_RBUTTONDOWN = 0x0204,
            WM_RBUTTONUP = 0x0205,
            WM_MOUSEMOVE = 0x0200,
            WM_MOUSEWHEEL = 0x020A
        }

        private delegate IntPtr MouseEventCallback(int nCode, IntPtr wParam, IntPtr lParam);

        private static MouseEventCallback? MouseEventCall;

        [DllImport("user32.dll", CharSet = CharSet.Auto, CallingConvention = CallingConvention.StdCall)]
        private static extern IntPtr SetWindowsHookEx(int idHook, MouseEventCallback lpfn, IntPtr hMod, uint dwThreadId);

        [DllImport("user32.dll", CharSet = CharSet.Auto, CallingConvention = CallingConvention.StdCall)]
        private static extern bool UnhookWindowsHookEx(IntPtr hhk);

        [DllImport("user32.dll", CharSet = CharSet.Auto, CallingConvention = CallingConvention.StdCall)]
        private static extern IntPtr CallNextHookEx(IntPtr hhk, int nCode, IntPtr wParam, IntPtr lParam);

        [StructLayout(LayoutKind.Sequential)]
        private struct MousePoint
        {
            public int X;
            public int Y;
        }

        [StructLayout(LayoutKind.Sequential)]
        private struct MouseHookStruct
        {
            public MousePoint pt;
            public IntPtr hwnd;
            public uint wHitTestCode;
            public IntPtr dwExtraInfo;
        }

        [StructLayout(LayoutKind.Sequential)]
        private struct MouseExtraHookStruct
        {
            public MousePoint Point;
            public int MouseData;
            public int Flags;
            public int Time;
            public IntPtr ExtraInfo;
        }

        private IntPtr CatchMouseEvent(int nCode, IntPtr wParam, IntPtr lParam)
        {
            if (State)
            {
                IBrowserHost WVHost = WallView.GetBrowser().GetHost();

                MouseExtraHookStruct mouseHookStruct = (MouseExtraHookStruct)Marshal.PtrToStructure(lParam, typeof(MouseExtraHookStruct));
                int X = mouseHookStruct.Point.X;
                int Y = mouseHookStruct.Point.Y;

                if (nCode >= 0 && MouseMessages.WM_MOUSEWHEEL == (MouseMessages)wParam)
                {
                    int delta = (mouseHookStruct.MouseData >> 16) & 0xFFFF;
                    bool isScrollDown = (delta & 0x8000) != 0;

                    int deltaX = mouseHookStruct.MouseData & 0xFFFF;
                    int deltaY = (mouseHookStruct.MouseData >> 16) & 0xFFFF;

                    int amount = 120;

                    if (isScrollDown)
                    {
                        //deltaX = -+(delta / amount);
                        deltaY = -amount;
                    }
                    else
                    {
                        //deltaX = delta / amount;
                        deltaY = amount;
                    }

                    MouseEvent mouseEvent = new(deltaX, deltaY, CefEventFlags.None);
                    WVHost.SendMouseWheelEvent(mouseEvent, deltaX, deltaY);
                }
                else if (nCode >= 0 && MouseMessages.WM_LBUTTONDOWN == (MouseMessages)wParam)
                {
                    // Sol tuşa basma olayı
                    // İlgili işlemleri burada gerçekleştirin
                    WVHost.SendMouseClickEvent(X, Y, MouseButtonType.Left, false, 1, CefEventFlags.None);
                }
                else if (nCode >= 0 && MouseMessages.WM_LBUTTONUP == (MouseMessages)wParam)
                {
                    // Sol tuştan el çekme olayı
                    // İlgili işlemleri burada gerçekleştirin
                    WVHost.SendMouseClickEvent(X, Y, MouseButtonType.Left, true, 1, CefEventFlags.None);
                }
                else if (nCode >= 0 && MouseMessages.WM_RBUTTONDOWN == (MouseMessages)wParam)
                {
                    // Sağ tuşa basma olayı
                    // İlgili işlemleri burada gerçekleştirin
                    WVHost.SendMouseClickEvent(X, Y, MouseButtonType.Right, false, 1, CefEventFlags.None);
                }
                else if (nCode >= 0 && MouseMessages.WM_RBUTTONUP == (MouseMessages)wParam)
                {
                    // Sağ tuştan el çekme olayı
                    // İlgili işlemleri burada gerçekleştirin
                    WVHost.SendMouseClickEvent(X, Y, MouseButtonType.Right, true, 1, CefEventFlags.None);
                }
                else if (nCode >= 0 && MouseMessages.WM_MOUSEMOVE == (MouseMessages)wParam)
                {
                    // Fare hareketi olayı
                    // İlgili işlemleri burada gerçekleştirin
                    WVHost.SendMouseMoveEvent(X, Y, false, CefEventFlags.None);
                }
            }

            return CallNextHookEx(IntPtr.Zero, nCode, wParam, lParam);
        }
    }

    public class CustomContextMenuHandler : IContextMenuHandler
    {
        public void OnBeforeContextMenu(IWebBrowser chromiumWebBrowser, IBrowser browser, IFrame frame, IContextMenuParams parameters, IMenuModel model)
        {
            model.Clear();
        }

        public bool OnContextMenuCommand(IWebBrowser chromiumWebBrowser, IBrowser browser, IFrame frame, IContextMenuParams parameters, CefMenuCommand commandId, CefEventFlags eventFlags)
        {
            return false;
        }

        public void OnContextMenuDismissed(IWebBrowser chromiumWebBrowser, IBrowser browser, IFrame frame)
        {
            return;
        }

        public bool RunContextMenu(IWebBrowser chromiumWebBrowser, IBrowser browser, IFrame frame, IContextMenuParams parameters, IMenuModel model, IRunContextMenuCallback callback)
        {
            return false;
        }
    }
}