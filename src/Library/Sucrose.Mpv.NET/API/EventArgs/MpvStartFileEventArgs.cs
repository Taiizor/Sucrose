namespace Sucrose.Mpv.NET.API
{
    public class MpvStartFileEventArgs : EventArgs
    {
        public MpvEventStartFile EventStartFile { get; private set; }

        public MpvStartFileEventArgs(MpvEventStartFile eventStartFile)
        {
            EventStartFile = eventStartFile;
        }
    }
}