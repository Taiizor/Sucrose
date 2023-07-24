using SEWTT = Skylark.Enum.WindowsThemeType;
using SMC = Sucrose.Memory.Constant;
using SMMI = Sucrose.Manager.Manage.Internal;
using SSLMI = Sucrose.Shared.Launcher.Manage.Internal;
using SSLVDRB = Sucrose.Shared.Launcher.View.DarkReportBox;
using SSLVLRB = Sucrose.Shared.Launcher.View.LightReportBox;
using SWHWT = Skylark.Wing.Helper.WindowsTheme;

namespace Sucrose.Shared.Launcher.Command
{
    internal static class Report
    {
        private static SEWTT Theme => SMMI.GeneralSettingManager.GetSetting(SMC.ThemeType, SWHWT.GetTheme());

        public static void Command()
        {
            if (SSLMI.ReportBox)
            {
                SSLMI.ReportBox = false;

                switch (Theme)
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