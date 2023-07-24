using SCHA = Sucrose.Commandog.Helper.Arguments;
using SSWW = Sucrose.Shared.Watchdog.Watch;

namespace Sucrose.Commandog
{
    internal class App
    {
        internal static void Main(string[] Args)
        {
            try
            {
                SCHA.Parse(Args);
            }
            catch (Exception Exception)
            {
                SSWW.Watch_CatchException(Exception);
            }
            finally
            {
                Close();
            }
        }

        protected static void Close()
        {
            Environment.Exit(0);
            Application.Exit();
        }
    }
}