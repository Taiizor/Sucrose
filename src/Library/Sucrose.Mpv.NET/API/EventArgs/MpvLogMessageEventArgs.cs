namespace Sucrose.Mpv.NET.API
{
    public class MpvLogMessageEventArgs : EventArgs
    {
        public MpvLogMessage Message { get; private set; }

        public MpvLogMessageEventArgs(MpvLogMessage message)
        {
            Message = message;
        }
    }
}