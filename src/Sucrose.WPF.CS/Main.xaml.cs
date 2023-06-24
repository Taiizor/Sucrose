using CefSharp;
using Skylark.Enum;
using Skylark.Struct.Monitor;
using Skylark.Wing.Helper;
using Skylark.Wing.Manage;
using Skylark.Wing.Native;
using Skylark.Wing.Utility;
using System.Runtime.InteropServices;
using System.Text;
using System.Windows;
using System.Windows.Threading;

namespace Sucrose.WPF.CS
{
    /// <summary>
    /// Interaction logic for Main.xaml
    /// </summary>
    public partial class Main : Window
    {
        private int ScreenIndex { get; set; } = 0;

        private bool IsFixed { get; set; } = false;

        private static WinAPI.MouseEventCallback? MouseEventCall;
        private static IntPtr MouseHook = IntPtr.Zero;

        private static WinAPI.WinEventDelegate? ForegroundDelegate;
        private static IntPtr ForegroundHook = IntPtr.Zero;

        private readonly DispatcherTimer Timer = new();

        public Main()
        {
            InitializeComponent();

            Variables.Hook = true;
            Variables.State = true;

            WallView.MenuHandler = new CustomContextMenuHandler();

            PinToBackground();

            Timer.Tick += new EventHandler(Timer_Tick);
            Timer.Interval = new TimeSpan(0, 0, 1);
            Timer.Start();

            ForegroundDelegate = new WinAPI.WinEventDelegate(FullScreenChanged);
            ForegroundHook = WinAPI.SetWinEventHook(External.EVENT_SYSTEM_FOREGROUND, External.EVENT_SYSTEM_FOREGROUND, IntPtr.Zero, ForegroundDelegate, 0, 0, External.WINEVENT_OUTOFCONTEXT);
        }

        protected void Timer_Tick(object sender, EventArgs e)
        {
            if (Variables.State)
            {
                Variables.State = false;
                WallView.Address = Variables.Uri;

                if (Variables.Hook)
                {
                    MouseEventCall = CatchMouseEvent;
                    MouseHook = WinAPI.SetWindowsHookEx(14, MouseEventCall, IntPtr.Zero, 0);
                }
                else
                {
                    MouseEventCall = null;
                    WinAPI.UnhookWinEvent(MouseHook);
                }
            }

            ForegroundAppMonitor();
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

        protected int OwnerScreenIndex
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

        protected MonitorStruct OwnerScreen
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
            try
            {
                IBrowserHost WVHost = WallView.GetBrowser().GetHost();

                MouseExtraHookStruct mouseHookStruct = (MouseExtraHookStruct)Marshal.PtrToStructure(lParam, typeof(MouseExtraHookStruct));
                int X = mouseHookStruct.Point.X;
                int Y = mouseHookStruct.Point.Y;

                if (nCode >= 0 && MouseMessagesType.WM_WHEEL == (MouseMessagesType)wParam)
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
                else if (nCode >= 0 && MouseMessagesType.WM_LBUTTONDOWN == (MouseMessagesType)wParam)
                {
                    // Sol tuşa basma olayı
                    // İlgili işlemleri burada gerçekleştirin
                    WVHost.SendMouseClickEvent(X, Y, MouseButtonType.Left, false, 1, CefEventFlags.None);
                }
                else if (nCode >= 0 && MouseMessagesType.WM_LBUTTONUP == (MouseMessagesType)wParam)
                {
                    // Sol tuştan el çekme olayı
                    // İlgili işlemleri burada gerçekleştirin
                    WVHost.SendMouseClickEvent(X, Y, MouseButtonType.Left, true, 1, CefEventFlags.None);
                }
                else if (nCode >= 0 && MouseMessagesType.WM_RBUTTONDOWN == (MouseMessagesType)wParam)
                {
                    // Sağ tuşa basma olayı
                    // İlgili işlemleri burada gerçekleştirin
                    WVHost.SendMouseClickEvent(X, Y, MouseButtonType.Right, false, 1, CefEventFlags.None);
                }
                else if (nCode >= 0 && MouseMessagesType.WM_RBUTTONUP == (MouseMessagesType)wParam)
                {
                    // Sağ tuştan el çekme olayı
                    // İlgili işlemleri burada gerçekleştirin
                    WVHost.SendMouseClickEvent(X, Y, MouseButtonType.Right, true, 1, CefEventFlags.None);
                }
                else if (nCode >= 0 && MouseMessagesType.WM_MOVE == (MouseMessagesType)wParam)
                {
                    // Fare hareketi olayı
                    // İlgili işlemleri burada gerçekleştirin
                    WVHost.SendMouseMoveEvent(X, Y, false, CefEventFlags.None);
                }

                return IntPtr.Zero;
                //return WinAPI.CallNextHookEx(IntPtr.Zero, nCode, wParam, lParam);
            }
            catch
            {
                return IntPtr.Zero;
            }
        }

        private void FullScreenChanged(IntPtr hWinEventHook, uint eventType, IntPtr hwnd, int idObject, int idChild, uint dwEventThread, uint dwmsEventTime)
        {
            try
            {
                if (hwnd != IntPtr.Zero)
                {
                    if (IsWhitelistedClass(hwnd)) //Remove this if you want to pin to background all windows
                    {
                        PinToBackground();
                    }
                }
            }
            catch
            {
                return;
            }
        }

        private readonly string[] ClassWhiteList = new string[]
        {
            //StartMeu, TaskView (Win10), action center etc
            "Windows.UI.Core.CoreWindow",
            //Alt+Tab Screen (Win10)
            "MultitaskingViewFrame",
            //TaskView (Win11)
            "XamlExplorerHostIslandWindow",
            "TaskListThumbnailWnd",
            //DirectX Fullscreen
            "ForegroundStaging",
            //Taskbar(s)
            "Shell_TrayWnd",
            "Shell_SecondaryTrayWnd",
            //Systray Notifyicon Expanded Popup
            "NotifyIconOverflowWindow",
            //RainMeter Widgets
            "RainmeterMeterWindow",
            //Coodesker
            "_cls_desk_"
        };

        private bool IsWhitelistedClass(IntPtr hwnd)
        {
            const int maxChars = 256;

            StringBuilder className = new(maxChars);

            return Methods.GetClassName((int)hwnd, className, maxChars) > 0 && ClassWhiteList.Any(x => x.Equals(className.ToString(), StringComparison.Ordinal));
        }

        private void ForegroundAppMonitor()
        {
            IntPtr fHandle = Methods.GetForegroundWindow();

            if (IsWhitelistedClass(fHandle))
            {
                PinToBackground();
                return;
            }
        }
    }

    internal class CustomContextMenuHandler : IContextMenuHandler
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