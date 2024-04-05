using System.Windows.Media;
using Wpf.Ui.Controls;
using SEDEST = Skylark.Enum.DuplicateScreenType;
using SEDYST = Skylark.Enum.DisplayScreenType;
using SEEST = Skylark.Enum.ExpandScreenType;
using SEST = Skylark.Enum.ScreenType;
using SEWTT = Skylark.Enum.WindowsThemeType;
using SMC = Sucrose.Memory.Constant;
using SMMI = Sucrose.Manager.Manage.Internal;
using SSCECT = Sucrose.Shared.Core.Enum.ChannelType;
using SSCEUT = Sucrose.Shared.Core.Enum.UpdateType;
using SSDECT = Sucrose.Shared.Dependency.Enum.CommunicationType;
using SSDEPPT = Sucrose.Shared.Dependency.Enum.PausePerformanceType;
using SSDEPT = Sucrose.Shared.Dependency.Enum.PerformanceType;
using SSDESKT = Sucrose.Shared.Dependency.Enum.SortKindType;
using SSDESMT = Sucrose.Shared.Dependency.Enum.SortModeType;
using SSDEST = Sucrose.Shared.Dependency.Enum.StretchType;
using SWHWT = Skylark.Wing.Helper.WindowsTheme;

namespace Sucrose.Portal.Manage
{
    internal static class Manager
    {
        public static SSDEPPT PausePerformanceType => SMMI.BackgroundogSettingManager.GetSetting(SMC.PausePerformanceType, SSDEPPT.Light);

        public static SSDEPT FullscreenPerformance => SMMI.BackgroundogSettingManager.GetSetting(SMC.FullscreenPerformance, SSDEPT.Close);

        public static Stretch BackgroundStretch => SMMI.PortalSettingManager.GetSetting(SMC.BackgroundStretch, DefaultBackgroundStretch);

        public static WindowBackdropType BackdropType => SMMI.PortalSettingManager.GetSetting(SMC.BackdropType, DefaultBackdropType);

        public static SSDEPT VirtualPerformance => SMMI.BackgroundogSettingManager.GetSetting(SMC.VirtualPerformance, SSDEPT.Resume);

        public static SSDEPT NetworkPerformance => SMMI.BackgroundogSettingManager.GetSetting(SMC.NetworkPerformance, SSDEPT.Resume);

        public static SSDEPT BatteryPerformance => SMMI.BackgroundogSettingManager.GetSetting(SMC.BatteryPerformance, SSDEPT.Close);

        public static SSDEPT RemotePerformance => SMMI.BackgroundogSettingManager.GetSetting(SMC.RemotePerformance, SSDEPT.Resume);

        public static SSDECT CommunicationType => SMMI.BackgroundogSettingManager.GetSetting(SMC.CommunicationType, SSDECT.Signal);

        public static SSDEPT MemoryPerformance => SMMI.BackgroundogSettingManager.GetSetting(SMC.MemoryPerformance, SSDEPT.Pause);

        public static SEDEST DuplicateScreenType => SMMI.EngineSettingManager.GetSetting(SMC.DuplicateScreenType, SEDEST.Default);

        public static SSDEPT FocusPerformance => SMMI.BackgroundogSettingManager.GetSetting(SMC.FocusPerformance, SSDEPT.Resume);

        public static SEDYST DisplayScreenType => SMMI.EngineSettingManager.GetSetting(SMC.DisplayScreenType, SEDYST.PerDisplay);

        public static SSDEPT SaverPerformance => SMMI.BackgroundogSettingManager.GetSetting(SMC.SaverPerformance, SSDEPT.Pause);

        public static SSDESKT LibrarySortKind => SMMI.PortalSettingManager.GetSetting(SMC.LibrarySortKind, SSDESKT.Descending);

        public static SSDEPT CpuPerformance => SMMI.BackgroundogSettingManager.GetSetting(SMC.CpuPerformance, SSDEPT.Close);

        public static SEEST ExpandScreenType => SMMI.EngineSettingManager.GetSetting(SMC.ExpandScreenType, SEEST.Default);

        public static SSDESMT LibrarySortMode => SMMI.PortalSettingManager.GetSetting(SMC.LibrarySortMode, SSDESMT.None);

        public static SSCEUT UpdateType => SMMI.UpdateSettingManager.GetSetting(SMC.UpdateType, SSCEUT.Compressed);

        public static SSCECT ChannelType => SMMI.UpdateSettingManager.GetSetting(SMC.ChannelType, SSCECT.Release);

        public static SEST ScreenType => SMMI.EngineSettingManager.GetSetting(SMC.ScreenType, SEST.DisplayBound);

        public static SEWTT ThemeType => SMMI.GeneralSettingManager.GetSetting(SMC.ThemeType, SWHWT.GetTheme());

        public static SSDEST StretchType => SMMI.EngineSettingManager.GetSetting(SMC.StretchType, SSDEST.Fill);

        public static WindowBackdropType DefaultBackdropType => WindowBackdropType.None;

        public static Stretch DefaultBackgroundStretch => Stretch.UniformToFill;
    }
}