namespace Sucrose.Mpv.NET.Player
{
    public class MpvPlayerPositionChangedEventArgs : EventArgs
    {
        public TimeSpan NewPosition { get; private set; }

        public MpvPlayerPositionChangedEventArgs(double newPosition)
        {
            NewPosition = TimeSpan.FromSeconds(newPosition);
        }
    }
}