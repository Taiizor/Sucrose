using SSLVDRB = Sucrose.Shared.Launcher.View.DarkReportBox;

namespace Sucrose.Shared.Launcher.Command
{
    internal static class Report
    {
        private static bool ShowBox = true;

        public static void Command()
        {
            if (ShowBox)
            {
                ShowBox = false;

                SSLVDRB Box = new();
                Box.ShowDialog();

                ShowBox = true;
            }
        }
    }
}