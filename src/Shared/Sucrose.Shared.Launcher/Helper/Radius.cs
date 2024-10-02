using System.Runtime.InteropServices;
using SWNM = Skylark.Wing.Native.Methods;

namespace Sucrose.Shared.Launcher.Helper
{
    internal static class Radius
    {
        public static void Corner(ContextMenuStrip ContextMenu)
        {
            try
            {
                SWNM.DWMWINDOWATTRIBUTE Attribute = SWNM.DWMWINDOWATTRIBUTE.WindowCornerPreference;
                SWNM.DWM_WINDOW_CORNER_PREFERENCE Preference = SWNM.DWM_WINDOW_CORNER_PREFERENCE.DWMWCP_ROUND;

                SWNM.DwmSetWindowAttribute(ContextMenu.Handle, Attribute, ref Preference, (uint)Marshal.SizeOf(typeof(uint)));
            }
            catch { }
        }
    }
}