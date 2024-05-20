using System.Runtime.InteropServices;

namespace Sucrose.Mpv.NET.API
{
    [StructLayout(LayoutKind.Sequential)]
    public struct MpvEventStartFile
    {
        public ulong PlaylistEntryId;
    }
}