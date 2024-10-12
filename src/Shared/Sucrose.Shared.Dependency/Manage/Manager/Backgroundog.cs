using SMMCB = Sucrose.Memory.Manage.Constant.Backgroundog;
using SMMI = Sucrose.Manager.Manage.Internal;
using SSDECT = Sucrose.Shared.Dependency.Enum.CommunicationType;
using SSDEPPT = Sucrose.Shared.Dependency.Enum.PausePerformanceType;
using SSDEPT = Sucrose.Shared.Dependency.Enum.PerformanceType;

namespace Sucrose.Shared.Dependency.Manage.Manager
{
    internal static class Backgroundog
    {
        public static SSDEPT FullscreenPerformance => SMMI.BackgroundogSettingManager.GetSetting(SMMCB.FullscreenPerformance, SSDEPT.Resume);

        public static SSDEPPT PausePerformanceType => SMMI.BackgroundogSettingManager.GetSetting(SMMCB.PausePerformanceType, SSDEPPT.Light);

        public static SSDEPT VirtualPerformance => SMMI.BackgroundogSettingManager.GetSetting(SMMCB.VirtualPerformance, SSDEPT.Resume);

        public static SSDEPT NetworkPerformance => SMMI.BackgroundogSettingManager.GetSetting(SMMCB.NetworkPerformance, SSDEPT.Resume);

        public static SSDEPT BatteryPerformance => SMMI.BackgroundogSettingManager.GetSetting(SMMCB.BatteryPerformance, SSDEPT.Resume);

        public static SSDEPT RemotePerformance => SMMI.BackgroundogSettingManager.GetSetting(SMMCB.RemotePerformance, SSDEPT.Resume);

        public static SSDEPT MemoryPerformance => SMMI.BackgroundogSettingManager.GetSetting(SMMCB.MemoryPerformance, SSDEPT.Resume);

        public static SSDEPT SaverPerformance => SMMI.BackgroundogSettingManager.GetSetting(SMMCB.SaverPerformance, SSDEPT.Resume);

        public static SSDEPT FocusPerformance => SMMI.BackgroundogSettingManager.GetSetting(SMMCB.FocusPerformance, SSDEPT.Resume);

        public static SSDECT CommunicationType => SMMI.BackgroundogSettingManager.GetSetting(SMMCB.CommunicationType, SSDECT.Pipe);

        public static SSDEPT GpuPerformance => SMMI.BackgroundogSettingManager.GetSetting(SMMCB.GpuPerformance, SSDEPT.Resume);

        public static SSDEPT CpuPerformance => SMMI.BackgroundogSettingManager.GetSetting(SMMCB.CpuPerformance, SSDEPT.Resume);
    }
}