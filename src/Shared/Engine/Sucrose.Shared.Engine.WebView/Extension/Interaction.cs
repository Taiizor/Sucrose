using SSEWVMI = Sucrose.Shared.Engine.WebView.Manage.Internal;
using SSDEIMT = Sucrose.Shared.Dependency.Enum.InputModuleType;
using SSDMM = Sucrose.Shared.Dependency.Manage.Manager;
using SMMM = Sucrose.Manager.Manage.Manager;
using SEIT = Skylark.Enum.InputType;
using SWNM = Skylark.Wing.Native.Methods;
using Linearstar.Windows.RawInput;
using System.Windows.Interop;
using SWHC = Skylark.Wing.Helper.Calculate;
using Point = System.Drawing.Point;
using Linearstar.Windows.RawInput.Native;
using SSEMI = Sucrose.Shared.Engine.Manage.Internal;

namespace Sucrose.Shared.Engine.WebView.Extension
{
    internal static class Interaction
    {
        public static void Register()
        {
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

            SSEWVMI.WebHandle = SSEWVMI.WebEngine.Handle;

            IntPtr InputHandle = SWNM.FindWindowEx(SSEWVMI.WebHandle, IntPtr.Zero, "Chrome_WidgetWin_0", null);

            if (!InputHandle.Equals(IntPtr.Zero))
            {
                SSEWVMI.WebHandle = SWNM.FindWindowEx(InputHandle, IntPtr.Zero, "Chrome_WidgetWin_1", null);
            }
        }

        public static void Unregister()
        {
            RawInputDevice.UnregisterDevice(HidUsageAndPage.Mouse);
            RawInputDevice.UnregisterDevice(HidUsageAndPage.Keyboard);
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

                SWNM.PostMessageW(SSEWVMI.WebHandle, Message, wParam, (UIntPtr)lParam);
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

                /* Same as:
                 * lParam = isPressed ? (lParam |= 0u << 30) : (lParam |= 1u << 30); //prev key state
                 * lParam = isPressed ? (lParam |= 0u << 31) : (lParam |= 1u << 31); //transition state
                */
                lParam = IsPressed ? lParam : (lParam |= 3u << 30);

                SWNM.PostMessageW(SSEWVMI.WebHandle, Message, wParam, (UIntPtr)lParam);
            }
            catch { }
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

                            switch (Mouse.Mouse.Buttons)
                            {
                                case RawMouseButtonFlags.MiddleButtonDown:
                                    ForwardMessageMouse(P.X, P.Y, (int)SWNM.WM.MBUTTONDOWN, (IntPtr)0x0010);
                                    break;
                                case RawMouseButtonFlags.MiddleButtonUp:
                                    ForwardMessageMouse(P.X, P.Y, (int)SWNM.WM.MBUTTONUP, (IntPtr)0x0010);
                                    break;
                                case RawMouseButtonFlags.LeftButtonDown:
                                    ForwardMessageMouse(P.X, P.Y, (int)SWNM.WM.LBUTTONDOWN, (IntPtr)0x0001);
                                    break;
                                case RawMouseButtonFlags.LeftButtonUp:
                                    ForwardMessageMouse(P.X, P.Y, (int)SWNM.WM.LBUTTONUP, (IntPtr)0x0001);
                                    break;
                                case RawMouseButtonFlags.RightButtonDown:
                                    //issue: click being skipped; desktop already has its own rightclick contextmenu.
                                    //ForwardMessage(M.X, M.Y, (int)SWNM.WM.RBUTTONDOWN, (IntPtr)0x0002);
                                    break;
                                case RawMouseButtonFlags.RightButtonUp:
                                    //issue: click being skipped; desktop already has its own rightclick contextmenu.
                                    //ForwardMessage(M.X, M.Y, (int)SWNM.WM.RBUTTONUP, (IntPtr)0x0002);
                                    break;
                                case RawMouseButtonFlags.None:
                                    ForwardMessageMouse(P.X, P.Y, (int)SWNM.WM.MOUSEMOVE, (IntPtr)0x0020);
                                    break;
                                case RawMouseButtonFlags.MouseWheel:
                                    //int MouseData = 7864320; //-7864320
                                    //int Delta = (MouseData >> 16) & 0xFFFF;

                                    //int Amount = Delta >> 15 == 1 ? Delta - 0xFFFF - 1 : Delta;

                                    //int DeltaX = MouseData & 0xFFFF;
                                    //int DeltaY = Amount;

                                    //SWNM.SendMessage(SSEWVMI.WebHandle, (int)SWNM.WM.MOUSEWHEEL, DeltaX, DeltaY);

                                    int MouseData = Mouse.Mouse.ButtonData;

                                    SWNM.SendMessage(SSEWVMI.WebHandle, (int)SWNM.WM.MOUSEWHEEL, IntPtr.Zero, (IntPtr)MouseData);
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
    }
}