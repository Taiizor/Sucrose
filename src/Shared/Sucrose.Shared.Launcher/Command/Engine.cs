using System.IO;
using SMMG = Sucrose.Manager.Manage.General;
using SMMI = Sucrose.Manager.Manage.Internal;
using SMML = Sucrose.Manager.Manage.Library;
using SSLCI = Sucrose.Shared.Launcher.Command.Interface;
using SSLHK = Sucrose.Shared.Live.Helper.Kill;
using SSLHR = Sucrose.Shared.Live.Helper.Run;
using SSSHL = Sucrose.Shared.Space.Helper.Live;

namespace Sucrose.Shared.Launcher.Command
{
    internal static class Engine
    {
        public static void Command(bool State = true)
        {
            if (State && SSSHL.Run())
            {
                SSLHK.Stop();
            }
            else if (!SSSHL.Run() && SMMI.LibrarySettingManager.CheckFile() && !string.IsNullOrEmpty(SMML.Selected))
            {
                if (Directory.Exists(Path.Combine(SMML.Location, SMML.Selected)))
                {
                    SSLHR.Start();

                    if (!SMMG.AppVisible)
                    {
                        SSLCI.Command();
                    }
                }
                else
                {
                    SSLCI.Command();
                }
            }
            else
            {
                SSLCI.Command();
            }
        }
    }
}