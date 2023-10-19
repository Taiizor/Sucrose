using SEWTT = Skylark.Enum.WindowsThemeType;
using SMC = Sucrose.Memory.Constant;
using SMMI = Sucrose.Manager.Manage.Internal;
using SWHWT = Skylark.Wing.Helper.WindowsTheme;

namespace Sucrose.Watchdog.Manage
{
    internal static class Manager
    {
        public static SEWTT ThemeType => SMMI.GeneralSettingManager.GetSetting(SMC.ThemeType, SWHWT.GetTheme());
    }
}