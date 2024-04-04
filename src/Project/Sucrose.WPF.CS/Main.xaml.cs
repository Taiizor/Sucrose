using CefSharp;
using Microsoft.Win32;
using Skylark.Enum;
using Skylark.Struct.Mouse;
using Skylark.Wing;
using Skylark.Wing.Helper;
using Skylark.Wing.Manage;
using Skylark.Wing.Native;
using Skylark.Wing.Utility;
using System.ComponentModel;
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
        private static WinAPI.MouseEventCallback MouseEventCall;
        private static IntPtr MouseHook = IntPtr.Zero;

        private static WinAPI.WinEventDelegate ForegroundDelegate;
        private static IntPtr ForegroundHook = IntPtr.Zero;

        private readonly DispatcherTimer Timer = new();

        private static IBrowserHost WVHost = null;

        public Main()
        {
            InitializeComponent();

            Variables.State = true;

            WallView.MenuHandler = new CustomContextMenuHandler();

            PinToBackground();

            Timer.Tick += new EventHandler(Timer_Tick);
            Timer.Interval = new TimeSpan(0, 0, 1);
            Timer.Start();

            SystemEvents.DisplaySettingsChanged += DisplaySettingsChanged;

            ForegroundDelegate = new WinAPI.WinEventDelegate(FullScreenChanged);
            ForegroundHook = WinAPI.SetWinEventHook(External.EVENT_SYSTEM_FOREGROUND, External.EVENT_SYSTEM_FOREGROUND, IntPtr.Zero, ForegroundDelegate, 0, 0, External.WINEVENT_OUTOFCONTEXT);
        }

        private void Window_Closing(object sender, CancelEventArgs e)
        {
            Desktop.RefreshDesktop();

            //WallView.Dispose();

            //Cef.Shutdown();
            //Cef.PreShutdown();
            //Cef.GetGlobalRequestContext()?.Dispose();
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
                    WVHost = WallView.GetBrowser().GetHost();
                    MouseHook = WinAPI.SetWindowsHookEx(14, MouseEventCall, IntPtr.Zero, 0);
                }
                else
                {
                    WVHost = null;
                    MouseEventCall = null;
                    WinAPI.UnhookWinEvent(MouseHook);
                }
            }

            ForegroundAppMonitor();
        }

        protected bool PinToBackground(int Index = 0, ScreenType Type = ScreenType.DisplayBound)
        {
            //return Engine.WallpaperWindow(this, DuplicateScreenType.Default, Type);
            return Engine.WallpaperWindow(this, ExpandScreenType.Default, Type);
            //return Engine.WallpaperWindow(this, Index, Type);
        }

        private IntPtr CatchMouseEvent(int nCode, IntPtr wParam, IntPtr lParam)
        {
            try
            {
                if (nCode >= 0 && WVHost != null)
                {
                    MouseExtraHookStruct HookStruct = (MouseExtraHookStruct)Marshal.PtrToStructure(lParam, typeof(MouseExtraHookStruct));

                    MousePointStruct Position = Calculate.MousePosition(HookStruct, DisplayScreenType.SpanAcross); //DisplayScreenType.PerDisplay

                    int X = Position.X;
                    int Y = Position.Y;

                    MouseMessagesType Type = (MouseMessagesType)wParam;

                    if (MouseMessagesType.WM_MBUTTONDOWN == Type)
                    {
                        WVHost.SendMouseClickEvent(X, Y, MouseButtonType.Middle, false, 1, CefEventFlags.None);
                    }
                    else if (MouseMessagesType.WM_MBUTTONUP == Type)
                    {
                        WVHost.SendMouseClickEvent(X, Y, MouseButtonType.Middle, true, 1, CefEventFlags.None);
                    }
                    else if (MouseMessagesType.WM_LBUTTONDOWN == Type)
                    {
                        WVHost.SendMouseClickEvent(X, Y, MouseButtonType.Left, false, 1, CefEventFlags.None);
                    }
                    else if (MouseMessagesType.WM_LBUTTONUP == Type)
                    {
                        WVHost.SendMouseClickEvent(X, Y, MouseButtonType.Left, true, 1, CefEventFlags.None);
                    }
                    else if (MouseMessagesType.WM_RBUTTONDOWN == Type)
                    {
                        WVHost.SendMouseClickEvent(X, Y, MouseButtonType.Right, false, 1, CefEventFlags.None);
                    }
                    else if (MouseMessagesType.WM_RBUTTONUP == Type)
                    {
                        WVHost.SendMouseClickEvent(X, Y, MouseButtonType.Right, true, 1, CefEventFlags.None);
                    }
                    else if (MouseMessagesType.WM_MOVE == Type)
                    {
                        WVHost.SendMouseMoveEvent(X, Y, false, CefEventFlags.None);
                    }
                    else if (MouseMessagesType.WM_WHEEL == Type)
                    {
                        int mouseData = HookStruct.MouseData;
                        int delta = (mouseData >> 16) & 0xFFFF;

                        int amount = delta >> 15 == 1 ? delta - 0xFFFF - 1 : delta;

                        int deltaX = mouseData & 0xFFFF;
                        int deltaY = amount;

                        MouseEvent MouseEvent = new(X, Y, CefEventFlags.None);
                        WVHost.SendMouseWheelEvent(MouseEvent, deltaX, deltaY);
                    }
                }

                return IntPtr.Zero;
                //return WinAPI.CallNextHookEx(IntPtr.Zero, nCode, wParam, lParam);
            }
            catch
            {
                return IntPtr.Zero;
            }
        }

        private void DisplaySettingsChanged(object sender, EventArgs e)
        {
            try
            {
                Screene.Initialize(); //Ekran boyutu değiştiyse güncelle

                PinToBackground();

            }
            catch
            {
                return;
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
}