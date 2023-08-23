using SMC = Sucrose.Memory.Constant;
using SMMI = Sucrose.Manager.Manage.Internal;
using SSLCI = Sucrose.Shared.Launcher.Command.Interface;
using SSLHR = Sucrose.Shared.Live.Helper.Run;
using SSSHL = Sucrose.Shared.Space.Helper.Live;
using SSSHP = Sucrose.Shared.Space.Helper.Processor;
using SWUD = Skylark.Wing.Utility.Desktop;

namespace Sucrose.Shared.Launcher.Command
{
    internal static class Engine
    {
        private static string LibrarySelected => SMMI.LibrarySettingManager.GetSetting(SMC.LibrarySelected, string.Empty);

        private static string App => SMMI.AuroraSettingManager.GetSetting(SMC.App, string.Empty);

        public static void Command(bool State = true)
        {
            if (State && SSSHL.Run())
            {
                SSSHL.Kill();

                if (!string.IsNullOrEmpty(App))
                {
                    SSSHP.Kill(App);
                }

                SWUD.RefreshDesktop();

                SMMI.AuroraSettingManager.SetSetting(SMC.App, string.Empty);
            }
            else if (!SSSHL.Run() && SMMI.EngineSettingManager.CheckFile() && !string.IsNullOrEmpty(LibrarySelected))
            {
                SSLHR.Start();
            }
            else if (!SMMI.EngineSettingManager.CheckFile() || string.IsNullOrEmpty(LibrarySelected))
            {
                SSLCI.Command();
            }
        }
    }
}