using System.Runtime.InteropServices;

namespace Sucrose.Mpv.NET.API
{
    [StructLayout(LayoutKind.Sequential)]
    public struct MpvEventClientMessage
    {
        public int NumArgs;

        public IntPtr Args;
    }
}