namespace Sucrose.Mpv.NET.API
{
    public class MpvEndFileEventArgs : EventArgs
    {
        public MpvEventEndFile EventEndFile { get; private set; }

        public MpvEndFileEventArgs(MpvEventEndFile eventEndFile)
        {
            EventEndFile = eventEndFile;
        }
    }
}