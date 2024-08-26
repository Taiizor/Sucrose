using SSWW = Sucrose.Shared.Watchdog.Watch;

namespace Sucrose.Shared.Space.Helper
{
    internal static class Exceptioner
    {
        public static async Task<string> GetMessage(Exception Error, string Empty, string Split)
        {
            try
            {
                if (Error == null)
                {
                    return Empty;
                }

                List<string> Messages = new();
                Exception CurrentException = Error;

                while (CurrentException != null)
                {
                    if (!string.IsNullOrWhiteSpace(CurrentException.Message))
                    {
                        Messages.Add(CurrentException.Message);
                    }

                    CurrentException = CurrentException.InnerException;
                }

                return Messages.Any() ? string.Join(Split, Messages) : Empty;
            }
            catch (Exception Exception)
            {
                await SSWW.Watch_CatchException(Exception);

                return Exception.Message;
            }
        }
    }
}