using System.Globalization;
using System.Text;
using SCHA = Sucrose.Commandog.Helper.Arguments;
using SHC = Skylark.Helper.Culture;
using SMMG = Sucrose.Manager.Manage.General;
using SSWW = Sucrose.Shared.Watchdog.Watch;

namespace Sucrose.Commandog
{
    internal class App : IDisposable
    {
        public static async Task Main(string[] Args)
        {
            try
            {
                Console.InputEncoding = Encoding.UTF8;
                Console.OutputEncoding = Encoding.UTF8;

                SHC.All = new CultureInfo(SMMG.Culture, true);

                await SCHA.Parse(Args);
            }
            catch (Exception Exception)
            {
                await SSWW.Watch_CatchException(Exception);
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