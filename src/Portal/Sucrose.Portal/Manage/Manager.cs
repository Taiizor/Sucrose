using System.Net;
using System.Windows.Media;
using Wpf.Ui.Controls;
using SEDST = Skylark.Enum.DuplicateScreenType;
using SEEST = Skylark.Enum.ExpandScreenType;
using SEST = Skylark.Enum.ScreenType;
using SEWTT = Skylark.Enum.WindowsThemeType;
using SHS = Skylark.Helper.Skymath;
using SMC = Sucrose.Memory.Constant;
using SMMI = Sucrose.Manager.Manage.Internal;
using SMR = Sucrose.Memory.Readonly;
using SSCEUT = Sucrose.Shared.Core.Enum.UpdateType;
using SSDEDT = Sucrose.Shared.Dependency.Enum.DisplayType;
using SSDEPT = Sucrose.Shared.Dependency.Enum.PerformanceType;
using SSDEST = Sucrose.Shared.Dependency.Enum.StretchType;
using SWHWT = Skylark.Wing.Helper.WindowsTheme;

namespace Sucrose.Portal.Manage
{
    internal static class Manager
    {
        public static SSDEPT FullscreenPerformance => SMMI.BackgroundogSettingManager.GetSetting(SMC.FullscreenPerformance, SSDEPT.Close);

        public static Stretch BackgroundStretch => SMMI.PortalSettingManager.GetSetting(SMC.BackgroundStretch, DefaultBackgroundStretch);

        public static WindowBackdropType BackdropType => SMMI.PortalSettingManager.GetSetting(SMC.BackdropType, DefaultBackdropType);

        public static SSDEPT VirtualPerformance => SMMI.BackgroundogSettingManager.GetSetting(SMC.VirtualPerformance, SSDEPT.Resume);

        public static SSDEPT NetworkPerformance => SMMI.BackgroundogSettingManager.GetSetting(SMC.NetworkPerformance, SSDEPT.Resume);

        public static SSDEPT BatteryPerformance => SMMI.BackgroundogSettingManager.GetSetting(SMC.BatteryPerformance, SSDEPT.Close);

        public static SSDEPT RemotePerformance => SMMI.BackgroundogSettingManager.GetSetting(SMC.RemotePerformance, SSDEPT.Resume);

        public static SSDEPT MemoryPerformance => SMMI.BackgroundogSettingManager.GetSetting(SMC.MemoryPerformance, SSDEPT.Pause);

        public static SSDEPT FocusPerformance => SMMI.BackgroundogSettingManager.GetSetting(SMC.FocusPerformance, SSDEPT.Resume);

        public static SEDST DuplicateScreenType => SMMI.EngineSettingManager.GetSetting(SMC.DuplicateScreenType, SEDST.Default);

        public static SSDEPT SaverPerformance => SMMI.BackgroundogSettingManager.GetSetting(SMC.SaverPerformance, SSDEPT.Pause);

        public static SSDEPT CpuPerformance => SMMI.BackgroundogSettingManager.GetSetting(SMC.CpuPerformance, SSDEPT.Close);

        public static SEEST ExpandScreenType => SMMI.EngineSettingManager.GetSetting(SMC.ExpandScreenType, SEEST.Default);

        public static IPAddress Host => SMMI.LauncherSettingManager.GetSettingAddress(SMC.Host, IPAddress.Loopback);

        public static SSCEUT UpdateType => SMMI.UpdateSettingManager.GetSetting(SMC.UpdateType, SSCEUT.Executable);

        public static SEST ScreenType => SMMI.EngineSettingManager.GetSetting(SMC.ScreenType, SEST.DisplayBound);

        public static int Port => SHS.Clamp(SMMI.LauncherSettingManager.GetSettingStable(SMC.Port, 0), 0, 65535);

        public static SSDEDT DisplayType => SMMI.EngineSettingManager.GetSetting(SMC.DisplayType, SSDEDT.Screen);

        public static SEWTT ThemeType => SMMI.GeneralSettingManager.GetSetting(SMC.ThemeType, SWHWT.GetTheme());

        public static SSDEST StretchType => SMMI.EngineSettingManager.GetSetting(SMC.StretchType, SSDEST.Fill);

        public static WindowBackdropType DefaultBackdropType => WindowBackdropType.None;

        public static Stretch DefaultBackgroundStretch => Stretch.UniformToFill;

        public static Mutex Mutex => new(true, SMR.PortalMutex);
    }
}