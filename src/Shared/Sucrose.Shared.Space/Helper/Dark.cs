using System.Runtime.InteropServices;
using SWNM = Skylark.Wing.Native.Methods;

namespace Sucrose.Shared.Space.Helper
{
    internal static class Dark
    {
        public static void Apply(IntPtr Handle)
        {
            try
            {
                bool Value = true;

                SWNM.DwmSetWindowAttribute(Handle, SWNM.DWMWindowAttribute.DWMWA_USE_IMMERSIVE_DARK_MODE, ref Value, Marshal.SizeOf(Value));
            }
            catch (Exception)
            {
                //
            }
        }
    }
}