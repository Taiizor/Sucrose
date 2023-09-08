using SELLT = Skylark.Enum.LevelLogType;
using SMC = Sucrose.Memory.Constant;
using SMMI = Sucrose.Manager.Manage.Internal;
using SMR = Sucrose.Memory.Readonly;
using SSSHL = Sucrose.Shared.Space.Helper.Live;
using SSSHP = Sucrose.Shared.Space.Helper.Processor;
using SWUD = Skylark.Wing.Utility.Desktop;
using WinForms = System.Windows.Forms.Application;

namespace Sucrose.Shared.Launcher.Command
{
    internal static class Close
    {
        private static string App => SMMI.AuroraSettingManager.GetSetting(SMC.App, string.Empty);

        private static bool Exit => SMMI.LauncherSettingManager.GetSetting(SMC.Exit, false);

        public static void Command()
        {
            SMMI.LauncherLogManager.Log(SELLT.Info, $"Application has been closed.");

            if (Exit)
            {
                if (SSSHL.Run())
                {
                    SSSHL.Kill();

                    if (!string.IsNullOrEmpty(App))
                    {
                        SSSHP.Kill(App);
                    }

                    SWUD.RefreshDesktop();

                    SMMI.AuroraSettingManager.SetSetting(SMC.App, string.Empty);
                }

                if (SSSHP.Work(SMR.Portal))
                {
                    SSSHP.Kill(SMR.Portal);
                }

                if (SSSHP.Work(SMR.Update))
                {
                    SSSHP.Kill(SMR.Update);
                }

                if (SSSHP.Work(SMR.Commandog))
                {
                    SSSHP.Kill(SMR.Commandog);
                }
            }

            WinForms.ExitThread();
            Environment.Exit(0);
            WinForms.Exit();
        }
    }
}