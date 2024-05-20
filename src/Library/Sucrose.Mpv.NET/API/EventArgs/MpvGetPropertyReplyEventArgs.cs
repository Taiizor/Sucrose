namespace Sucrose.Mpv.NET.API
{
    public class MpvGetPropertyReplyEventArgs : EventArgs
    {
        public ulong ReplyUserData { get; private set; }

        public MpvError Error { get; private set; }

        public MpvEventProperty EventProperty { get; private set; }

        public MpvGetPropertyReplyEventArgs(ulong replyUserData, MpvError error, MpvEventProperty eventProperty)
        {
            ReplyUserData = replyUserData;
            Error = error;
            EventProperty = eventProperty;
        }
    }
}