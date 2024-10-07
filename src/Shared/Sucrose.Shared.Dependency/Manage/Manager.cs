using System.Windows.Media;
using SEWTT = Skylark.Enum.WindowsThemeType;
using SMC = Sucrose.Memory.Constant;
using SMMI = Sucrose.Manager.Manage.Internal;
using SSDECT = Sucrose.Shared.Dependency.Enum.CommunicationType;
using SSDEET = Sucrose.Shared.Dependency.Enum.EngineType;
using SSDEIMT = Sucrose.Shared.Dependency.Enum.InputModuleType;
using SSDEPPT = Sucrose.Shared.Dependency.Enum.PausePerformanceType;
using SSDEPT = Sucrose.Shared.Dependency.Enum.PerformanceType;
using SSDESHT = Sucrose.Shared.Dependency.Enum.StretchType;
using SSDESKT = Sucrose.Shared.Dependency.Enum.SortKindType;
using SSDESMT = Sucrose.Shared.Dependency.Enum.SortModeType;
using SSDESST = Sucrose.Shared.Dependency.Enum.StoreServerType;
using SSDETCT = Sucrose.Shared.Dependency.Enum.TransitionCycleType;
using SSDEUMT = Sucrose.Shared.Dependency.Enum.UpdateModuleType;
using SSDEUST = Sucrose.Shared.Dependency.Enum.UpdateServerType;
using SSSMI = Sucrose.Shared.Space.Manage.Internal;
using SWHWT = Skylark.Wing.Helper.WindowsTheme;

namespace Sucrose.Shared.Dependency.Manage
{
    internal static class Manager
    {
        public static SSDEET ApplicationEngine => SMMI.EngineSettingManager.GetSetting(SMC.ApplicationEngine, (SSDEET)SSSMI.ApplicationEngine);

        public static SSDEPT FullscreenPerformance => SMMI.BackgroundogSettingManager.GetSetting(SMC.FullscreenPerformance, SSDEPT.Resume);

        public static SSDEPPT PausePerformanceType => SMMI.BackgroundogSettingManager.GetSetting(SMC.PausePerformanceType, SSDEPPT.Light);

        public static Stretch BackgroundStretch => SMMI.PortalSettingManager.GetSetting(SMC.BackgroundStretch, DefaultBackgroundStretch);

        public static SSDEPT VirtualPerformance => SMMI.BackgroundogSettingManager.GetSetting(SMC.VirtualPerformance, SSDEPT.Resume);

        public static SSDEPT NetworkPerformance => SMMI.BackgroundogSettingManager.GetSetting(SMC.NetworkPerformance, SSDEPT.Resume);

        public static SSDEPT BatteryPerformance => SMMI.BackgroundogSettingManager.GetSetting(SMC.BatteryPerformance, SSDEPT.Resume);

        public static SSDETCT TransitionCycleType => SMMI.CyclingSettingManager.GetSetting(SMC.TransitionCycleType, SSDETCT.Random);

        public static SSDEET YouTubeEngine => SMMI.EngineSettingManager.GetSetting(SMC.YouTubeEngine, (SSDEET)SSSMI.YouTubeEngine);

        public static SSDEPT RemotePerformance => SMMI.BackgroundogSettingManager.GetSetting(SMC.RemotePerformance, SSDEPT.Resume);

        public static SSDEPT MemoryPerformance => SMMI.BackgroundogSettingManager.GetSetting(SMC.MemoryPerformance, SSDEPT.Resume);

        public static SSDEUMT UpdateModuleType => SMMI.UpdateSettingManager.GetSetting(SMC.UpdateModuleType, SSDEUMT.Downloader);

        public static SSDEPT SaverPerformance => SMMI.BackgroundogSettingManager.GetSetting(SMC.SaverPerformance, SSDEPT.Resume);

        public static SSDEPT FocusPerformance => SMMI.BackgroundogSettingManager.GetSetting(SMC.FocusPerformance, SSDEPT.Resume);

        public static SSDECT CommunicationType => SMMI.BackgroundogSettingManager.GetSetting(SMC.CommunicationType, SSDECT.Pipe);

        public static SSDEUST UpdateServerType => SMMI.UpdateSettingManager.GetSetting(SMC.UpdateServerType, SSDEUST.Soferity);

        public static SSDESKT LibrarySortKind => SMMI.PortalSettingManager.GetSetting(SMC.LibrarySortKind, SSDESKT.Descending);

        public static SSDEET VideoEngine => SMMI.EngineSettingManager.GetSetting(SMC.VideoEngine, (SSDEET)SSSMI.VideoEngine);

        public static SSDESST StoreServerType => SMMI.PortalSettingManager.GetSetting(SMC.StoreServerType, SSDESST.Soferity);

        public static SSDESMT LibrarySortMode => SMMI.PortalSettingManager.GetSetting(SMC.LibrarySortMode, SSDESMT.Creation);

        public static SSDEIMT InputModuleType => SMMI.EngineSettingManager.GetSetting(SMC.InputModuleType, SSDEIMT.RawInput);

        public static SSDEPT GpuPerformance => SMMI.BackgroundogSettingManager.GetSetting(SMC.GpuPerformance, SSDEPT.Resume);

        public static SSDEPT CpuPerformance => SMMI.BackgroundogSettingManager.GetSetting(SMC.CpuPerformance, SSDEPT.Resume);

        public static SSDEET WebEngine => SMMI.EngineSettingManager.GetSetting(SMC.WebEngine, (SSDEET)SSSMI.WebEngine);

        public static SSDEET UrlEngine => SMMI.EngineSettingManager.GetSetting(SMC.UrlEngine, (SSDEET)SSSMI.UrlEngine);

        public static SSDEET GifEngine => SMMI.EngineSettingManager.GetSetting(SMC.GifEngine, (SSDEET)SSSMI.GifEngine);

        public static SSDESHT StretchType => SMMI.EngineSettingManager.GetSetting(SMC.StretchType, SSDESHT.Fill);

        public static SEWTT ThemeType => SMMI.GeneralSettingManager.GetSetting(SMC.ThemeType, SWHWT.GetTheme());

        public static Stretch DefaultBackgroundStretch => Stretch.UniformToFill;

        public static SSDESHT DefaultStretchType => SSDESHT.None;
    }
}