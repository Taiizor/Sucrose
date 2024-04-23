using Wpf.Ui.Controls;
using SEWTT = Skylark.Enum.WindowsThemeType;
using SMC = Sucrose.Memory.Constant;
using SMMI = Sucrose.Manager.Manage.Internal;
using SWHWT = Skylark.Wing.Helper.WindowsTheme;

namespace Sucrose.Property.Manage
{
    internal static class Manager
    {
        public static WindowBackdropType BackdropType => SMMI.PortalSettingManager.GetSetting(SMC.BackdropType, DefaultBackdropType);

        public static SEWTT ThemeType => SMMI.GeneralSettingManager.GetSetting(SMC.ThemeType, SWHWT.GetTheme());

        public static WindowBackdropType DefaultBackdropType => WindowBackdropType.None;
    }
}