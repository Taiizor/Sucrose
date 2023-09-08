using SEDST = Skylark.Enum.DuplicateScreenType;
using SEEST = Skylark.Enum.ExpandScreenType;
using SEST = Skylark.Enum.ScreenType;
using SEWTT = Skylark.Enum.WindowsThemeType;
using SMC = Sucrose.Memory.Constant;
using SMMI = Sucrose.Manager.Manage.Internal;
using SMR = Sucrose.Memory.Readonly;
using SSDEDT = Sucrose.Shared.Dependency.Enum.DisplayType;
using SSDEST = Sucrose.Shared.Dependency.Enum.StretchType;
using SWHWT = Skylark.Wing.Helper.WindowsTheme;

namespace Sucrose.Shared.Engine.Manage
{
    internal static class Manager
    {
        public static SEDST DuplicateScreenType => SMMI.EngineSettingManager.GetSetting(SMC.DuplicateScreenType, SEDST.Default);

        public static SEEST ExpandScreenType => SMMI.EngineSettingManager.GetSetting(SMC.ExpandScreenType, SEEST.Default);

        public static SEST ScreenType => SMMI.EngineSettingManager.GetSetting(SMC.ScreenType, SEST.DisplayBound);

        public static SSDEDT DisplayType => SMMI.EngineSettingManager.GetSetting(SMC.DisplayType, SSDEDT.Screen);

        public static SSDEST StretchType => SMMI.EngineSettingManager.GetSetting(SMC.StretchType, SSDEST.Fill);

        public static SEWTT Theme => SMMI.GeneralSettingManager.GetSetting(SMC.ThemeType, SWHWT.GetTheme());

        public static SSDEST DefaultStretchType => SSDEST.None;

        public static Mutex Mutex => new(true, SMR.LiveMutex);
    }
}