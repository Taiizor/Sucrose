using SELLT = Skylark.Enum.LevelLogType;
using SMMI = Sucrose.Manager.Manage.Internal;
using WinForms = System.Windows.Forms.Application;

namespace Sucrose.Shared.Launcher.Command
{
    internal static class Close
    {
        public static void Command()
        {
            SMMI.LauncherLogManager.Log(SELLT.Info, $"Application has been closed.");

            WinForms.ExitThread();
            Environment.Exit(0);
            WinForms.Exit();
        }
    }
}