using SMC = Sucrose.Memory.Constant;
using SMMI = Sucrose.Manager.Manage.Internal;
using SMR = Sucrose.Memory.Readonly;
using SSECT = Sucrose.Space.Enum.CommandsType;
using SSHC = Sucrose.Space.Helper.Command;
using SSHL = Sucrose.Space.Helper.Live;
using SSMI = Sucrose.Space.Manage.Internal;
using SWUD = Skylark.Wing.Utility.Desktop;

namespace Sucrose.Tray.Command
{
    internal static class Engine
    {
        private static string Live => SMMI.EngineSettingManager.GetSetting(SMC.App, SMR.EngineLive);

        public static void Command(bool State = true)
        {
            if (State && SSHL.Run())
            {
                SSHL.Kill();

                SWUD.RefreshDesktop();
            }
            else if (SMMI.EngineSettingManager.CheckFile())
            {
                SSHC.Run(SSMI.CommandLine, $"{SMR.StartCommand}{SSECT.Live}{SMR.ValueSeparator}{SSMI.TextEngineLive[Live]}");
            }
        }
    }
}