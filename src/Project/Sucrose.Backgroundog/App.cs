using System.Globalization;
using System.Text;
using SBMI = Sucrose.Backgroundog.Manage.Internal;
using SBMM = Sucrose.Backgroundog.Manage.Manager;
using SGCB = Sucrose.Grpc.Common.Backgroundog;
using SGSGSS = Sucrose.Grpc.Services.GeneralServerService;
using SHC = Skylark.Helper.Culture;
using SMC = Sucrose.Memory.Constant;
using SMMI = Sucrose.Manager.Manage.Internal;
using SMMM = Sucrose.Manager.Manage.Manager;
using SMR = Sucrose.Memory.Readonly;
using SSSHP = Sucrose.Shared.Space.Helper.Processor;
using SSSSBSS = Sucrose.Shared.Server.Services.BackgroundogServerService;
using SSWW = Sucrose.Shared.Watchdog.Watch;

namespace Sucrose.Backgroundog
{
    internal class App : IDisposable
    {
        public static async Task Main()
        {
            try
            {
                Console.OutputEncoding = Encoding.UTF8;
                SHC.All = new CultureInfo(SMMM.Culture, true);

                if (SBMM.Mutex.WaitOne(TimeSpan.Zero, true) && SSSHP.WorkCount(SMR.Backgroundog) <= 1)
                {
                    SBMM.Mutex.ReleaseMutex();

                    Console.WriteLine("Start");

                    SGSGSS.ServerCreate(SGCB.BindService(new SSSSBSS()));

                    SMMI.BackgroundogSettingManager.SetSetting(SMC.Host, SGSGSS.Host);
                    SMMI.BackgroundogSettingManager.SetSetting(SMC.Port, SGSGSS.Port);

                    SGSGSS.ServerInstance.Start();

                    SBMI.Initialize.Start();

                    do
                    {
                        SBMI.Initialize.Dispose();

                        await Task.Delay(1000);
                    } while (SBMI.Exit);

                    SBMI.Initialize.Stop();

                    SGSGSS.ServerInstance.KillAsync().Wait();
                    //SGSGSS.ServerInstance.ShutdownAsync().Wait();

                    Console.WriteLine("Stop");
                }
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