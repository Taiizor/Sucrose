using SMC = Sucrose.Memory.Constant;
using SMMI = Sucrose.Manager.Manage.Internal;
using SSDEPPT = Sucrose.Shared.Dependency.Enum.PausePerformanceType;
using SSDEPT = Sucrose.Shared.Dependency.Enum.PerformanceType;

namespace Sucrose.Backgroundog.Manage
{
    internal static class Manager
    {
        public static SSDEPPT PausePerformanceType => SMMI.BackgroundogSettingManager.GetSetting(SMC.PausePerformanceType, SSDEPPT.Light);

        public static SSDEPT FullscreenPerformance => SMMI.BackgroundogSettingManager.GetSetting(SMC.FullscreenPerformance, SSDEPT.Close);

        public static SSDEPT VirtualPerformance => SMMI.BackgroundogSettingManager.GetSetting(SMC.VirtualPerformance, SSDEPT.Resume);

        public static SSDEPT NetworkPerformance => SMMI.BackgroundogSettingManager.GetSetting(SMC.NetworkPerformance, SSDEPT.Resume);

        public static SSDEPT BatteryPerformance => SMMI.BackgroundogSettingManager.GetSetting(SMC.BatteryPerformance, SSDEPT.Pause);

        public static SSDEPT RemotePerformance => SMMI.BackgroundogSettingManager.GetSetting(SMC.RemotePerformance, SSDEPT.Resume);

        public static SSDEPT MemoryPerformance => SMMI.BackgroundogSettingManager.GetSetting(SMC.MemoryPerformance, SSDEPT.Pause);

        public static SSDEPT FocusPerformance => SMMI.BackgroundogSettingManager.GetSetting(SMC.FocusPerformance, SSDEPT.Resume);

        public static SSDEPT SaverPerformance => SMMI.BackgroundogSettingManager.GetSetting(SMC.SaverPerformance, SSDEPT.Pause);

        public static SSDEPT CpuPerformance => SMMI.BackgroundogSettingManager.GetSetting(SMC.CpuPerformance, SSDEPT.Pause);

        public static bool Windows11_OrGreater => Environment.OSVersion.Version.Build >= 22000;
    }
}