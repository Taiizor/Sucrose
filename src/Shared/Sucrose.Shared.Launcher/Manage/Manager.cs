using SEWTT = Skylark.Enum.WindowsThemeType;
using SMC = Sucrose.Memory.Constant;
using SMMI = Sucrose.Manager.Manage.Internal;
using SMR = Sucrose.Memory.Readonly;
using SWHWT = Skylark.Wing.Helper.WindowsTheme;

namespace Sucrose.Shared.Launcher.Manage
{
    internal static class Manager
    {
        public static SEWTT ThemeType => SMMI.GeneralSettingManager.GetSetting(SMC.ThemeType, SWHWT.GetTheme());

        public static Mutex Mutex => new(true, SMR.LauncherMutex);
    }
}