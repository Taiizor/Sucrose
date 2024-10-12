using SEWTT = Skylark.Enum.WindowsThemeType;
using SMMCG = Sucrose.Memory.Manage.Constant.General;
using SMMI = Sucrose.Manager.Manage.Internal;
using SWHWT = Skylark.Wing.Helper.WindowsTheme;

namespace Sucrose.Shared.Dependency.Manage.Manager
{
    internal static class General
    {
        public static SEWTT ThemeType => SMMI.GeneralSettingManager.GetSetting(SMMCG.ThemeType, SWHWT.GetTheme());
    }
}