using System.Net;
using SEDST = Skylark.Enum.DuplicateScreenType;
using SEEST = Skylark.Enum.ExpandScreenType;
using SEST = Skylark.Enum.ScreenType;
using SEWTT = Skylark.Enum.WindowsThemeType;
using SHS = Skylark.Helper.Skymath;
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

        public static IPAddress Host => SMMI.BackgroundogSettingManager.GetSettingAddress(SMC.Host, IPAddress.Loopback);

        public static int Port => SHS.Clamp(SMMI.BackgroundogSettingManager.GetSettingStable(SMC.Port, 0), 0, 65535);

        public static SEST ScreenType => SMMI.EngineSettingManager.GetSetting(SMC.ScreenType, SEST.DisplayBound);

        public static SSDEDT DisplayType => SMMI.EngineSettingManager.GetSetting(SMC.DisplayType, SSDEDT.Screen);

        public static SEWTT ThemeType => SMMI.GeneralSettingManager.GetSetting(SMC.ThemeType, SWHWT.GetTheme());

        public static SSDEST StretchType => SMMI.EngineSettingManager.GetSetting(SMC.StretchType, SSDEST.Fill);

        public static SSDEST DefaultStretchType => SSDEST.None;

        public static Mutex Mutex => new(true, SMR.LiveMutex);
    }
}