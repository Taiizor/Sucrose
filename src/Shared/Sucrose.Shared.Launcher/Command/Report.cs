using SEWTT = Skylark.Enum.WindowsThemeType;
using SSLMI = Sucrose.Shared.Launcher.Manage.Internal;
using SSLMM = Sucrose.Shared.Launcher.Manage.Manager;
using SSLVDRB = Sucrose.Shared.Launcher.View.DarkReportBox;
using SSLVLRB = Sucrose.Shared.Launcher.View.LightReportBox;

namespace Sucrose.Shared.Launcher.Command
{
    internal static class Report
    {
        public static void Command()
        {
            if (SSLMI.ReportBox)
            {
                SSLMI.ReportBox = false;

                switch (SSLMM.Theme)
                {
                    case SEWTT.Dark:
                        SSLVDRB DarkReportBox = new();
                        DarkReportBox.ShowDialog();
                        break;
                    default:
                        SSLVLRB LightReportBox = new();
                        LightReportBox.ShowDialog();
                        break;
                }

                SSLMI.ReportBox = true;
            }
        }
    }
}