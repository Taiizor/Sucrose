using SEWTT = Skylark.Enum.WindowsThemeType;
using SSDMM = Sucrose.Shared.Dependency.Manage.Manager;
using SSLMI = Sucrose.Shared.Launcher.Manage.Internal;
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

                switch (SSDMM.ThemeType)
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