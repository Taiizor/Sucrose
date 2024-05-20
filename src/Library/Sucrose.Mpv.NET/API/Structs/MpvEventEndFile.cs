using System.Runtime.InteropServices;

namespace Sucrose.Mpv.NET.API
{
    [StructLayout(LayoutKind.Sequential)]
    public struct MpvEventEndFile
    {
        public MpvEndFileReason Reason;

        public MpvError Error;

        public ulong PlaylistEntryId;

        public int PlaylistInsertNumEntries;
    }
}