using CefSharp;
using Linearstar.Windows.RawInput;
using Linearstar.Windows.RawInput.Native;
using System.Windows.Interop;
using Point = System.Drawing.Point;
using SEIT = Skylark.Enum.InputType;
using SMMM = Sucrose.Manager.Manage.Manager;
using SSDEIMT = Sucrose.Shared.Dependency.Enum.InputModuleType;
using SSDMM = Sucrose.Shared.Dependency.Manage.Manager;
using SSECSMI = Sucrose.Shared.Engine.CefSharp.Manage.Internal;
using SSEMI = Sucrose.Shared.Engine.Manage.Internal;
using SWHC = Skylark.Wing.Helper.Calculate;
using SWNM = Skylark.Wing.Native.Methods;

namespace Sucrose.Shared.Engine.CefSharp.Extension
{
    internal static class Interaction
    {
        public static void Register()
        {
            if (SSEMI.Interaction)
            {
                SSEMI.Interaction = false;

                IntPtr HWND = SSEMI.WindowHandle;

                switch (SSDMM.InputModuleType)
                {
                    case SSDEIMT.Native:
                        break;
                    case SSDEIMT.RawInput:
                        if (SMMM.InputType is SEIT.OnlyMouse or SEIT.MouseKeyboard)
                        {
                            RawInputDevice.RegisterDevice(HidUsageAndPage.Mouse, RawInputDeviceFlags.ExInputSink, HWND);
                        }

                        if (SMMM.InputType is SEIT.OnlyKeyboard or SEIT.MouseKeyboard)
                        {
                            RawInputDevice.RegisterDevice(HidUsageAndPage.Keyboard, RawInputDeviceFlags.ExInputSink, HWND);
                        }

                        HwndSource Source = HwndSource.FromHwnd(HWND);
                        Source.AddHook(Hook);

                        break;
                }

                SSECSMI.CefHost = SSECSMI.CefEngine.GetBrowserHost();

                SSECSMI.CefHandle = SSECSMI.CefHost.GetWindowHandle();

                IntPtr InputHandle = SWNM.FindWindowEx(SSECSMI.CefHandle, IntPtr.Zero, "Chrome_WidgetWin_0", null);

                if (!InputHandle.Equals(IntPtr.Zero))
                {
                    SSECSMI.CefHandle = InputHandle;
                }

                if (SMMM.InputType is SEIT.OnlyKeyboard)
                {
                    //SSECSMI.CefHost?.SendMouseClickEvent(99999, 99999, MouseButtonType.Left, false, 1, CefEventFlags.None);
                    //SSECSMI.CefHost?.SendMouseClickEvent(99999, 99999, MouseButtonType.Left, true, 1, CefEventFlags.None);
                    ForwardMessageMouse(99999, 99999, (int)SWNM.WM.LBUTTONDOWN, (IntPtr)0x0001);
                    ForwardMessageMouse(99999, 99999, (int)SWNM.WM.LBUTTONUP, (IntPtr)0x0001);
                }
            }
        }

        public static void Unregister()
        {
            SSEMI.Interaction = true;

            RawInputDevice.UnregisterDevice(HidUsageAndPage.Mouse);
            RawInputDevice.UnregisterDevice(HidUsageAndPage.Keyboard);
        }

        private static IntPtr Hook(IntPtr HWND, int Message, IntPtr wParam, IntPtr lParam, ref bool Handled)
        {
            try
            {
                if (Message == (int)SWNM.WM.INPUT)
                {
                    RawInputData Data = RawInputData.FromHandle(lParam);

                    switch (Data)
                    {
                        case RawInputMouseData Mouse:
                            if (!SWNM.GetCursorPos(out SWNM.POINT P))
                            {
                                break;
                            }

                            Point Position = SWHC.MousePosition(P.X, P.Y, SMMM.DisplayScreenType);

                            switch (Mouse.Mouse.Buttons)
                            {
                                case RawMouseButtonFlags.MiddleButtonDown:
                                    ForwardMessageMouse(P.X, P.Y, (int)SWNM.WM.MBUTTONDOWN, (IntPtr)0x0010);
                                    //SSECSMI.CefHost?.SendMouseClickEvent(Position.X, Position.Y, MouseButtonType.Middle, false, 1, CefEventFlags.None);
                                    break;
                                case RawMouseButtonFlags.MiddleButtonUp:
                                    ForwardMessageMouse(P.X, P.Y, (int)SWNM.WM.MBUTTONUP, (IntPtr)0x0010);
                                    //SSECSMI.CefHost?.SendMouseClickEvent(Position.X, Position.Y, MouseButtonType.Middle, true, 1, CefEventFlags.None);
                                    break;
                                case RawMouseButtonFlags.LeftButtonDown:
                                    ForwardMessageMouse(P.X, P.Y, (int)SWNM.WM.LBUTTONDOWN, (IntPtr)0x0001);
                                    //SSECSMI.CefHost?.SendMouseClickEvent(Position.X, Position.Y, MouseButtonType.Left, false, 1, CefEventFlags.None);
                                    break;
                                case RawMouseButtonFlags.LeftButtonUp:
                                    ForwardMessageMouse(P.X, P.Y, (int)SWNM.WM.LBUTTONUP, (IntPtr)0x0001);
                                    //SSECSMI.CefHost?.SendMouseClickEvent(Position.X, Position.Y, MouseButtonType.Left, true, 1, CefEventFlags.None);
                                    break;
                                case RawMouseButtonFlags.RightButtonDown:
                                    //issue: click being skipped; desktop already has its own rightclick contextmenu.
                                    //ForwardMessageMouse(P.X, P.Y, (int)SWNM.WM.RBUTTONDOWN, (IntPtr)0x0002);
                                    //SSECSMI.CefHost?.SendMouseClickEvent(Position.X, Position.Y, MouseButtonType.Right, false, 1, CefEventFlags.None);
                                    break;
                                case RawMouseButtonFlags.RightButtonUp:
                                    //issue: click being skipped; desktop already has its own rightclick contextmenu.
                                    //ForwardMessageMouse(P.X, P.Y, (int)SWNM.WM.RBUTTONUP, (IntPtr)0x0002);
                                    //SSECSMI.CefHost?.SendMouseClickEvent(Position.X, Position.Y, MouseButtonType.Right, true, 1, CefEventFlags.None);
                                    break;
                                case RawMouseButtonFlags.None:
                                    ForwardMessageMouse(P.X, P.Y, (int)SWNM.WM.MOUSEMOVE, (IntPtr)0x0020);
                                    //SSECSMI.CefHost?.SendMouseMoveEvent(Position.X, Position.Y, false, CefEventFlags.None);
                                    break;
                                case RawMouseButtonFlags.MouseWheel:
                                    int MouseData = Mouse.Mouse.ButtonData;

                                    MouseEvent MouseEvent = new(Position.X, Position.Y, CefEventFlags.None);
                                    SSECSMI.CefHost?.SendMouseWheelEvent(MouseEvent, 0, MouseData);
                                    break;
                            }
                            break;
                        case RawInputKeyboardData Keyboard:
                            ForwardMessageKeyboard((int)Keyboard.Keyboard.WindowMessage, (IntPtr)Keyboard.Keyboard.VirutalKey, Keyboard.Keyboard.ScanCode, Keyboard.Keyboard.Flags != RawKeyboardFlags.Up);
                            break;
                    }
                }
            }
            catch { }

            return IntPtr.Zero;
        }

        private static void ForwardMessageMouse(int X, int Y, int Message, IntPtr wParam)
        {
            try
            {
                Point Mouse = SWHC.MousePosition(X, Y, SMMM.DisplayScreenType);

                //The low-order word specifies the x-coordinate of the cursor, the high-order word specifies the y-coordinate of the cursor.
                //ref: https://docs.microsoft.com/en-us/windows/win32/inputdev/wm-mousemove

                uint lParam = Convert.ToUInt32(Mouse.Y);

                lParam <<= 16;
                lParam |= Convert.ToUInt32(Mouse.X);

                SWNM.PostMessageW(SSECSMI.CefHandle, Message, wParam, (UIntPtr)lParam);
            }
            catch { }
        }

        private static void ForwardMessageKeyboard(int Message, IntPtr wParam, int ScanCode, bool IsPressed)
        {
            try
            {
                //ref:
                //https://docs.microsoft.com/en-us/windows/win32/inputdev/wm-keydown
                //https://docs.microsoft.com/en-us/windows/win32/inputdev/wm-keyup

                uint lParam = 1u; //press

                lParam |= (uint)ScanCode << 16; //oem code
                lParam |= 1u << 24; //extended key
                lParam |= 0u << 29; //context code; Note: Alt key combos wont't work

                /*
                 * Same as:
                 * lParam = isPressed ? (lParam |= 0u << 30) : (lParam |= 1u << 30); //prev key state
                 * lParam = isPressed ? (lParam |= 0u << 31) : (lParam |= 1u << 31); //transition state
                */
                lParam = IsPressed ? lParam : (lParam |= 3u << 30);

                //SSECSMI.CefHost?.SendKeyEvent(Message, (int)wParam, (int)lParam);
                SWNM.PostMessageW(SSECSMI.CefHandle, Message, wParam, (UIntPtr)lParam);
            }
            catch { }
        }
    }
}