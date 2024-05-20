namespace Sucrose.Mpv.NET.API
{
    public class MpvSetPropertyReplyEventArgs : EventArgs
    {
        public ulong ReplyUserData { get; private set; }

        public MpvError Error { get; private set; }

        public MpvSetPropertyReplyEventArgs(ulong replyUserData, MpvError error)
        {
            this.ReplyUserData = replyUserData;
            Error = error;
        }
    }
}