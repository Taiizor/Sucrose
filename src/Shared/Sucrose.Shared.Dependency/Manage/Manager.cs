using System.Windows.Media;
using SEWTT = Skylark.Enum.WindowsThemeType;
using SMC = Sucrose.Memory.Constant;
using SMMI = Sucrose.Manager.Manage.Internal;
using SSDECT = Sucrose.Shared.Dependency.Enum.CommunicationType;
using SSDEET = Sucrose.Shared.Dependency.Enum.EngineType;
using SSDEIMT = Sucrose.Shared.Dependency.Enum.InputModuleType;
using SSDEPPT = Sucrose.Shared.Dependency.Enum.PausePerformanceType;
using SSDEPT = Sucrose.Shared.Dependency.Enum.PerformanceType;
using SSDESET = Sucrose.Shared.Dependency.Enum.StoreType;
using SSDESHT = Sucrose.Shared.Dependency.Enum.StretchType;
using SSDESKT = Sucrose.Shared.Dependency.Enum.SortKindType;
using SSDESMT = Sucrose.Shared.Dependency.Enum.SortModeType;
using SSDETT = Sucrose.Shared.Dependency.Enum.TransitionType;
using SSSMI = Sucrose.Shared.Space.Manage.Internal;
using SWHWT = Skylark.Wing.Helper.WindowsTheme;

namespace Sucrose.Shared.Dependency.Manage
{
    internal static class Manager
    {
        public static SSDEPT FullscreenPerformance => SMMI.BackgroundogSettingManager.GetSetting(SMC.FullscreenPerformance, SSDEPT.Resume);

        public static SSDEPPT PausePerformanceType => SMMI.BackgroundogSettingManager.GetSetting(SMC.PausePerformanceType, SSDEPPT.Light);

        public static Stretch BackgroundStretch => SMMI.PortalSettingManager.GetSetting(SMC.BackgroundStretch, DefaultBackgroundStretch);

        public static SSDEPT VirtualPerformance => SMMI.BackgroundogSettingManager.GetSetting(SMC.VirtualPerformance, SSDEPT.Resume);

        public static SSDEPT NetworkPerformance => SMMI.BackgroundogSettingManager.GetSetting(SMC.NetworkPerformance, SSDEPT.Resume);

        public static SSDEPT BatteryPerformance => SMMI.BackgroundogSettingManager.GetSetting(SMC.BatteryPerformance, SSDEPT.Resume);

        public static SSDEPT RemotePerformance => SMMI.BackgroundogSettingManager.GetSetting(SMC.RemotePerformance, SSDEPT.Resume);

        public static SSDEPT MemoryPerformance => SMMI.BackgroundogSettingManager.GetSetting(SMC.MemoryPerformance, SSDEPT.Resume);

        public static SSDECT CommunicationType => SMMI.BackgroundogSettingManager.GetSetting(SMC.CommunicationType, SSDECT.Signal);

        public static SSDEPT SaverPerformance => SMMI.BackgroundogSettingManager.GetSetting(SMC.SaverPerformance, SSDEPT.Resume);

        public static SSDEPT FocusPerformance => SMMI.BackgroundogSettingManager.GetSetting(SMC.FocusPerformance, SSDEPT.Resume);

        public static SSDESKT LibrarySortKind => SMMI.PortalSettingManager.GetSetting(SMC.LibrarySortKind, SSDESKT.Descending);

        public static SSDESMT LibrarySortMode => SMMI.PortalSettingManager.GetSetting(SMC.LibrarySortMode, SSDESMT.Creation);

        public static SSDEIMT InputModuleType => SMMI.EngineSettingManager.GetSetting(SMC.InputModuleType, SSDEIMT.RawInput);

        public static SSDEPT CpuPerformance => SMMI.BackgroundogSettingManager.GetSetting(SMC.CpuPerformance, SSDEPT.Resume);

        public static SSDEET AApp => SMMI.EngineSettingManager.GetSetting(SMC.AApp, (SSDEET)SSSMI.ApplicationEngine);

        public static SSDETT TransitionType => SMMI.CyclingManager.GetSetting(SMC.TransitionType, SSDETT.Random);

        public static SSDESHT StretchType => SMMI.EngineSettingManager.GetSetting(SMC.StretchType, SSDESHT.Fill);

        public static SSDESET StoreType => SMMI.PortalSettingManager.GetSetting(SMC.StoreType, SSDESET.Soferity);

        public static SSDEET YApp => SMMI.EngineSettingManager.GetSetting(SMC.YApp, (SSDEET)SSSMI.YouTubeEngine);

        public static SEWTT ThemeType => SMMI.GeneralSettingManager.GetSetting(SMC.ThemeType, SWHWT.GetTheme());

        public static SSDEET VApp => SMMI.EngineSettingManager.GetSetting(SMC.VApp, (SSDEET)SSSMI.VideoEngine);

        public static SSDEET GApp => SMMI.EngineSettingManager.GetSetting(SMC.GApp, (SSDEET)SSSMI.GifEngine);

        public static SSDEET UApp => SMMI.EngineSettingManager.GetSetting(SMC.UApp, (SSDEET)SSSMI.UrlEngine);

        public static SSDEET WApp => SMMI.EngineSettingManager.GetSetting(SMC.WApp, (SSDEET)SSSMI.WebEngine);

        public static Stretch DefaultBackgroundStretch => Stretch.UniformToFill;

        public static SSDESHT DefaultStretchType => SSDESHT.None;
    }
}