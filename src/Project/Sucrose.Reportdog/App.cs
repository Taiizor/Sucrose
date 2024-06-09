using System.Globalization;
using System.Text;
using SHC = Skylark.Helper.Culture;
using SMMM = Sucrose.Manager.Manage.Manager;
using SMR = Sucrose.Memory.Readonly;
using SRHA = Sucrose.Reportdog.Helper.Attempt;
using SRMI = Sucrose.Reportdog.Manage.Internal;
using SSSHI = Sucrose.Shared.Space.Helper.Instance;
using SSSHS = Sucrose.Shared.Space.Helper.Security;
using SSWW = Sucrose.Shared.Watchdog.Watch;

namespace Sucrose.Reportdog
{
    internal class App : IDisposable
    {
        public static async Task Main()
        {
            try
            {
                Console.InputEncoding = Encoding.UTF8;
                Console.OutputEncoding = Encoding.UTF8;

                SHC.All = new CultureInfo(SMMM.Culture, true);

                if (SSSHI.Basic(SMR.ReportdogMutex, SMR.Reportdog) && (SMMM.Report || SMMM.Statistics))
                {
                    SSSHS.Apply();

                    SRMI.Initialize.Start();

                    do
                    {
                        await SRHA.Start();

                        SRMI.Initialize.Dispose();

                        await Task.Delay(SRMI.AppTime);
                    } while (SRMI.Exit);

                    SRMI.Initialize.Stop();
                }
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