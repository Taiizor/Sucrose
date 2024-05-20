namespace Sucrose.Mpv.NET.API
{
    public class MpvCommandReplyEventArgs : EventArgs
    {
        public ulong ReplyUserData { get; private set; }

        public MpvError Error { get; private set; }

        public MpvCommandReplyEventArgs(ulong replyUserData, MpvError error)
        {
            ReplyUserData = replyUserData;
            Error = error;
        }
    }
}