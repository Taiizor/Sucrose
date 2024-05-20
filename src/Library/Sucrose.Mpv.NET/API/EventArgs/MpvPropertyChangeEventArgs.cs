namespace Sucrose.Mpv.NET.API
{
    public class MpvPropertyChangeEventArgs : EventArgs
    {
        public ulong ReplyUserData { get; private set; }

        public MpvEventProperty EventProperty { get; private set; }

        public MpvPropertyChangeEventArgs(ulong replyUserData, MpvEventProperty eventProperty)
        {
            ReplyUserData = replyUserData;
            EventProperty = eventProperty;
        }
    }
}