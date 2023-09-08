using SMC = Sucrose.Memory.Constant;
using SMMI = Sucrose.Manager.Manage.Internal;
using SMMM = Sucrose.Manager.Manage.Manager;
using SSLCI = Sucrose.Shared.Launcher.Command.Interface;
using SSLHR = Sucrose.Shared.Live.Helper.Run;
using SSSHL = Sucrose.Shared.Space.Helper.Live;
using SSSHP = Sucrose.Shared.Space.Helper.Processor;
using SWUD = Skylark.Wing.Utility.Desktop;

namespace Sucrose.Shared.Launcher.Command
{
    internal static class Engine
    {
        public static void Command(bool State = true)
        {
            if (State && SSSHL.Run())
            {
                SSSHL.Kill();

                if (!string.IsNullOrEmpty(SMMM.App))
                {
                    SSSHP.Kill(SMMM.App);
                }

                SWUD.RefreshDesktop();

                SMMI.AuroraSettingManager.SetSetting(SMC.App, string.Empty);
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