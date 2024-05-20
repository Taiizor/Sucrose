namespace Sucrose.Mpv.NET.API
{
    public class MpvAPIException : Exception
    {
        public MpvError Error { get; private set; }

        public static MpvAPIException FromError(MpvError error, IMpvFunctions functions)
        {
            string errorString = functions.ErrorString(error);

            string message = $"Error occured: \"{errorString}\".";

            return new MpvAPIException(message, error);
        }

        public MpvAPIException(string message, MpvError error) : base(message)
        {
            Error = error;
        }

        public MpvAPIException(string message) : base(message)
        {
        }
    }
}