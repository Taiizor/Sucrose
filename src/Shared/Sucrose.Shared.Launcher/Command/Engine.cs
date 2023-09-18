using SMMI = Sucrose.Manager.Manage.Internal;
using SMMM = Sucrose.Manager.Manage.Manager;
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
            else if (!SSSHL.Run() && SMMI.LibrarySettingManager.CheckFile() && !string.IsNullOrEmpty(SMMM.LibrarySelected))
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
    }
}