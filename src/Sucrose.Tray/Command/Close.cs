using WinForms = System.Windows.Forms.Application;

namespace Sucrose.Tray.Command
{
    public static class Close
    {
        public static void Command()
        {
            WinForms.ExitThread();
            Environment.Exit(0);
            WinForms.Exit();
        }
    }
}