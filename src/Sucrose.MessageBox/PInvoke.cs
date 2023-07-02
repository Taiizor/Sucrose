using System.Runtime.InteropServices;

namespace Sucrose.MessageBox
{
    public static class PInvoke
    {
        [DllImport("dwmapi.dll", PreserveSig = true)]
        public static extern int DwmSetWindowAttribute(IntPtr hwnd, WMMWindowAttribute attr, ref bool attrValue, int attrSize);
    }
}
