namespace Sucrose.Mpv.NET.Player
{
    public class MpvPlayerException : Exception
    {
        public MpvPlayerException()
        {
        }

        public MpvPlayerException(string message) : base(message)
        {
        }

        public MpvPlayerException(string message, Exception innerException) : base(message, innerException)
        {
        }
    }
}