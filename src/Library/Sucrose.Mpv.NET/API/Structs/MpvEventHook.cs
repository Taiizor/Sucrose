using System.Runtime.InteropServices;

namespace Sucrose.Mpv.NET.API
{
    [StructLayout(LayoutKind.Sequential)]
    public struct MpvEventHook
    {
        public IntPtr Name;

        public ulong Id;
    }
}