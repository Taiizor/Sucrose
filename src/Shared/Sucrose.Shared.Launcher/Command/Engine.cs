using System.IO;
using SMMI = Sucrose.Manager.Manage.Internal;
using SMMM = Sucrose.Manager.Manage.Manager;
using SSLCI = Sucrose.Shared.Launcher.Command.Interface;
using SSLHK = Sucrose.Shared.Live.Helper.Kill;
using SSLHR = Sucrose.Shared.Live.Helper.Run;
using SSSHL = Sucrose.Shared.Space.Helper.Live;
using SMML = Sucrose.Manager.Manage.Library;
using SMMCL = Sucrose.Memory.Manage.Constant.Library;

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
            else if (!SSSHL.Run() && SMMI.LibrarySettingManager.CheckFile() && !string.IsNullOrEmpty(SMML.LibrarySelected))
            {
                if (Directory.Exists(Path.Combine(SMML.LibraryLocation, SMML.LibrarySelected)))
                {
                    SSLHR.Start();

                    if (!SMMM.Visible)
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