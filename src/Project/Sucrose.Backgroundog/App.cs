﻿using System.Globalization;
using System.Text;
using SBMI = Sucrose.Backgroundog.Manage.Internal;
using SHC = Skylark.Helper.Culture;
using SMMG = Sucrose.Manager.Manage.General;
using SMMRA = Sucrose.Memory.Manage.Readonly.App;
using SMMRM = Sucrose.Memory.Manage.Readonly.Mutex;
using SSSHI = Sucrose.Shared.Space.Helper.Instance;
using SSSHS = Sucrose.Shared.Space.Helper.Security;
using SSWEW = Sucrose.Shared.Watchdog.Extension.Watch;

namespace Sucrose.Backgroundog
{
    internal class App : IDisposable
    {
        public static async Task Main()
        {
            try
            {
                Console.InputEncoding = Encoding.UTF8;
                Console.OutputEncoding = Encoding.UTF8;

                SHC.All = new CultureInfo(SMMG.Culture, true);

                if (SSSHI.Basic(SMMRM.Backgroundog, SMMRA.Backgroundog))
                {
                    SSSHS.Apply();

                    SBMI.Initialize.Start();

                    do
                    {
                        SBMI.Initialize.Dispose();

                        await Task.Delay(SBMI.AppTime);
                    } while (SBMI.Exit);

                    SBMI.Initialize.Stop();
                }
            }
            catch (Exception Exception)
            {
                await SSWEW.Watch_CatchException(Exception);
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