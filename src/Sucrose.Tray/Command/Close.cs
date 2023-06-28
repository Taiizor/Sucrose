using SCMI = Sucrose.Common.Manage.Internal;
using SELLT = Skylark.Enum.LevelLogType;
using WinForms = System.Windows.Forms.Application;

namespace Sucrose.Tray.Command
{
    public static class Close
    {
        public static void Command()
        {
            SCMI.TrayIconLogManager.Log(SELLT.Info, $"Application has been closed.");

            WinForms.ExitThread();
            Environment.Exit(0);
            WinForms.Exit();
        }
    }
}