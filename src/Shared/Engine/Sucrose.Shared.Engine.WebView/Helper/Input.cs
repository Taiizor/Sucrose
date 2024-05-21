using SSEMI = Sucrose.Shared.Engine.Manage.Internal;
using SSEWVMI = Sucrose.Shared.Engine.WebView.Manage.Internal;
using SWNM = Skylark.Wing.Native.Methods;

namespace Sucrose.Shared.Engine.WebView.Helper
{
    internal static class Handle
    {
        public static void GetInputHandle()
        {
            SSEWVMI.WebHandle = SSEWVMI.WebEngine.Handle;

            IntPtr InputHandle = SWNM.FindWindowEx(SSEWVMI.WebHandle, IntPtr.Zero, "Chrome_WidgetWin_0", null);

            if (!InputHandle.Equals(IntPtr.Zero))
            {
                SSEWVMI.WebHandle = SWNM.FindWindowEx(InputHandle, IntPtr.Zero, "Chrome_WidgetWin_1", null);
            }
        }

        public static void GetIntermediateHandle()
        {
            _ = SWNM.GetWindowThreadProcessId(SWNM.FindWindowEx(SSEWVMI.WebHandle, IntPtr.Zero, "Intermediate D3D Window", null), out SSEMI.IntermediateD3DWindow);
        }
    }
}