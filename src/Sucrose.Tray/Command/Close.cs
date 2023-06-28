using Skylark.Enum;
using Sucrose.Common.Manage;
using WinForms = System.Windows.Forms.Application;

namespace Sucrose.Tray.Command
{
    public static class Close
    {
        public static void Command()
        {
            Internal.TrayIconLogManager.Log(LevelLogType.Info, $"Application has been closed.");

            WinForms.ExitThread();
            Environment.Exit(0);
            WinForms.Exit();
        }
    }
}