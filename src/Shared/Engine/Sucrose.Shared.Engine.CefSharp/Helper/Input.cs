using CefSharp;
using SSECSMI = Sucrose.Shared.Engine.CefSharp.Manage.Internal;
using SSEMI = Sucrose.Shared.Engine.Manage.Internal;
using SWNM = Skylark.Wing.Native.Methods;

namespace Sucrose.Shared.Engine.CefSharp.Helper
{
    internal static class Handle
    {
        public static void GetInputHandle()
        {
            SSECSMI.CefHost = SSECSMI.CefEngine.GetBrowserHost();

            SSECSMI.CefHandle = SSECSMI.CefHost.GetWindowHandle();

            IntPtr InputHandle = SWNM.FindWindowEx(SSECSMI.CefHandle, IntPtr.Zero, "Chrome_WidgetWin_0", null);

            if (!InputHandle.Equals(IntPtr.Zero))
            {
                SSECSMI.CefHandle = InputHandle;
            }
        }

        public static void GetIntermediateHandle()
        {
            _ = SWNM.GetWindowThreadProcessId(SWNM.FindWindowEx(SSECSMI.CefHandle, IntPtr.Zero, "Intermediate D3D Window", null), out SSEMI.IntermediateD3DWindow);
        }
    }
}