using SELLT = Skylark.Enum.LevelLogType;
using SMMI = Sucrose.Manager.Manage.Internal;
using SMMM = Sucrose.Manager.Manage.Manager;
using SMR = Sucrose.Memory.Readonly;
using SSLHK = Sucrose.Shared.Live.Helper.Kill;
using SSSHL = Sucrose.Shared.Space.Helper.Live;
using SSSHP = Sucrose.Shared.Space.Helper.Processor;
using WinForms = System.Windows.Forms.Application;

namespace Sucrose.Shared.Launcher.Command
{
    internal static class Close
    {
        public static void Command()
        {
            SMMI.LauncherLogManager.Log(SELLT.Info, $"Application has been closed.");

            if (SMMM.Exit)
            {
                if (SSSHL.Run())
                {
                    SSLHK.Stop();
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