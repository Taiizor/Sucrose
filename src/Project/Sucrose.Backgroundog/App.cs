using System.Globalization;
using System.Text;
using SBMI = Sucrose.Backgroundog.Manage.Internal;
using SGCB = Sucrose.Grpc.Common.Backgroundog;
using SGSGSS = Sucrose.Grpc.Services.GeneralServerService;
using SHC = Skylark.Helper.Culture;
using SMC = Sucrose.Memory.Constant;
using SMMI = Sucrose.Manager.Manage.Internal;
using SMMM = Sucrose.Manager.Manage.Manager;
using SMR = Sucrose.Memory.Readonly;
using SSSHI = Sucrose.Shared.Space.Helper.Instance;
using SSSSBSS = Sucrose.Shared.Signal.Services.BackgroundogServerService;
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

                if (SSSHI.Basic(SMR.BackgroundogMutex, SMR.Backgroundog))
                {
                    SGSGSS.ServerCreate(SGCB.BindService(new SSSSBSS()));

                    SMMI.BackgroundogSettingManager.SetSetting(SMC.Host, SGSGSS.Host);
                    SMMI.BackgroundogSettingManager.SetSetting(SMC.Port, SGSGSS.Port);

                    SGSGSS.ServerInstance.Start();

                    SBMI.Initialize.Start();

                    do
                    {
                        SBMI.Initialize.Dispose();

                        await Task.Delay(SBMI.AppTime);
                    } while (SBMI.Exit);

                    SBMI.Initialize.Stop();

                    SGSGSS.ServerInstance.KillAsync().Wait();
                    //SGSGSS.ServerInstance.ShutdownAsync().Wait();
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