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

            if (SMMM.AppExit)
            {
                if (SSSHL.Run())
                {
                    SSLHK.Stop();
                }
                else
                {
                    SSLHK.StopSubprocess();
                }

                if (SSSHP.Work(SMR.Undo))
                {
                    SSSHP.Kill(SMR.Undo);
                }

                if (SSSHP.Work(SMR.Portal))
                {
                    SSSHP.Kill(SMR.Portal);
                }

                if (SSSHP.Work(SMR.Update))
                {
                    SSSHP.Kill(SMR.Update);
                }

                if (SSSHP.Work(SMR.Property))
                {
                    SSSHP.Kill(SMR.Property);
                }

                if (SSSHP.Work(SMR.Watchdog))
                {
                    SSSHP.Kill(SMR.Watchdog);
                }

                if (SSSHP.Work(SMR.Commandog))
                {
                    SSSHP.Kill(SMR.Commandog);
                }

                if (SSSHP.Work(SMR.Reportdog))
                {
                    SSSHP.Kill(SMR.Reportdog);
                }

                if (SSSHP.Work(SMR.Backgroundog))
                {
                    SSSHP.Kill(SMR.Backgroundog);
                }
            }

            WinForms.ExitThread();
            Environment.Exit(0);
            WinForms.Exit();
        }
    }
}