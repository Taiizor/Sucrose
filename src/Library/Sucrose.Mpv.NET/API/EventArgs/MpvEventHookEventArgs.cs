namespace Sucrose.Mpv.NET.API
{
    public class MpvEventHookEventArgs : EventArgs
    {
        public MpvEventHook EventHook { get; private set; }

        public MpvEventHookEventArgs(MpvEventHook eventHook)
        {
            EventHook = eventHook;
        }
    }
}