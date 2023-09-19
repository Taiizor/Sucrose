using System.Globalization;
using SCHA = Sucrose.Commandog.Helper.Arguments;
using SHC = Skylark.Helper.Culture;
using SMMM = Sucrose.Manager.Manage.Manager;
using SSWW = Sucrose.Shared.Watchdog.Watch;

namespace Sucrose.Commandog
{
    internal class App : IDisposable
    {
        public static void Main(string[] Args)
        {
            try
            {
                SHC.All = new CultureInfo(SMMM.Culture, true);

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

        public static void Close()
        {
            Environment.Exit(0);
            Application.Exit();
        }

        public void Dispose()
        {
            GC.Collect();
            GC.SuppressFinalize(this);
        }
    }
}