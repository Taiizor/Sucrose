using SEWTT = Skylark.Enum.WindowsThemeType;
using SMC = Sucrose.Memory.Constant;
using SMMI = Sucrose.Manager.Manage.Internal;
using SSLVDRB = Sucrose.Shared.Launcher.View.DarkReportBox;
using SWHWT = Skylark.Wing.Helper.WindowsTheme;

namespace Sucrose.Shared.Launcher.Command
{
    internal static class Report
    {
        private static SEWTT Theme => SMMI.GeneralSettingManager.GetSetting(SMC.ThemeType, SWHWT.GetTheme());

        private static bool ShowBox = true;

        public static void Command()
        {
            if (ShowBox)
            {
                ShowBox = false;

                switch (Theme)
                {
                    case SEWTT.Dark:
                        SSLVDRB DarkReportBox = new();
                        DarkReportBox.ShowDialog();
                        break;
                    default:
                        break;
                }

                ShowBox = true;
            }
        }
    }
}