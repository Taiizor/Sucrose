using SELLT = Skylark.Enum.LevelLogType;
using SMMI = Sucrose.Manager.Manage.Internal;
using SMMM = Sucrose.Manager.Manage.Manager;
using SMMRA = Sucrose.Memory.Manage.Readonly.App;
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

                if (SSSHP.Work(SMMRA.Undo))
                {
                    SSSHP.Kill(SMMRA.Undo);
                }

                if (SSSHP.Work(SMMRA.Portal))
                {
                    SSSHP.Kill(SMMRA.Portal);
                }

                if (SSSHP.Work(SMMRA.Update))
                {
                    SSSHP.Kill(SMMRA.Update);
                }

                if (SSSHP.Work(SMMRA.Property))
                {
                    SSSHP.Kill(SMMRA.Property);
                }

                if (SSSHP.Work(SMMRA.Watchdog))
                {
                    SSSHP.Kill(SMMRA.Watchdog);
                }

                if (SSSHP.Work(SMMRA.Commandog))
                {
                    SSSHP.Kill(SMMRA.Commandog);
                }

                if (SSSHP.Work(SMMRA.Reportdog))
                {
                    SSSHP.Kill(SMMRA.Reportdog);
                }

                if (SSSHP.Work(SMMRA.Backgroundog))
                {
                    SSSHP.Kill(SMMRA.Backgroundog);
                }
            }

            WinForms.ExitThread();
            Environment.Exit(0);
            WinForms.Exit();
        }
    }
}