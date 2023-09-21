using SMC = Sucrose.Memory.Constant;
using SMMI = Sucrose.Manager.Manage.Internal;
using SMR = Sucrose.Memory.Readonly;
using SSDEPT = Sucrose.Shared.Dependency.Enum.PerformanceType;

namespace Sucrose.Backgroundog.Manage
{
    internal static class Manager
    {
        public static SSDEPT NetworkPerformance => SMMI.BackgroundogSettingManager.GetSetting(SMC.NetworkPerformance, SSDEPT.Resume);

        public static SSDEPT BatteryPerformance => SMMI.BackgroundogSettingManager.GetSetting(SMC.BatteryPerformance, SSDEPT.Pause);

        public static SSDEPT MemoryPerformance => SMMI.BackgroundogSettingManager.GetSetting(SMC.MemoryPerformance, SSDEPT.Pause);

        public static SSDEPT SaverPerformance => SMMI.BackgroundogSettingManager.GetSetting(SMC.SaverPerformance, SSDEPT.Pause);

        public static SSDEPT CpuPerformance => SMMI.BackgroundogSettingManager.GetSetting(SMC.CpuPerformance, SSDEPT.Pause);

        public static bool Windows11_OrGreater => Environment.OSVersion.Version.Build >= 22000;

        public static Mutex Mutex => new(true, SMR.BackgroundogMutex);
    }
}