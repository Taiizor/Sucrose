using SEDEST = Skylark.Enum.DuplicateScreenType;
using SEDYST = Skylark.Enum.DisplayScreenType;
using SEEST = Skylark.Enum.ExpandScreenType;
using SEIT = Skylark.Enum.InputType;
using SESET = Skylark.Enum.StorageType;
using SESNT = Skylark.Enum.ScreenType;
using SHC = Skylark.Helper.Culture;
using SHS = Skylark.Helper.Skymath;
using SMC = Sucrose.Memory.Constant;
using SMMI = Sucrose.Manager.Manage.Internal;
using SMR = Sucrose.Memory.Readonly;
using SMMRP = Sucrose.Memory.Manage.Readonly.Path;
using SMMCH = Sucrose.Memory.Manage.Constant.Hook;
using SMMCU = Sucrose.Memory.Manage.Constant.User;
using SMMCA = Sucrose.Memory.Manage.Constant.Aurora;
using SMMCD = Sucrose.Memory.Manage.Constant.Donate;
using SMMCB = Sucrose.Memory.Manage.Constant.Backgroundog;

namespace Sucrose.Manager.Manage
{
    public static class Backgroundog
    {
        public static int DownloadValue => SHS.Clamp(SMMI.BackgroundogSettingManager.GetSettingStable(SMMCB.DownloadValue, 10), 0, 99999999);

        public static int UploadValue => SHS.Clamp(SMMI.BackgroundogSettingManager.GetSettingStable(SMMCB.UploadValue, 800), 0, 99999999);

        public static int BatteryUsage => SHS.Clamp(SMMI.BackgroundogSettingManager.GetSettingStable(SMMCB.BatteryUsage, 50), 0, 100);

        public static int MemoryUsage => SHS.Clamp(SMMI.BackgroundogSettingManager.GetSettingStable(SMMCB.MemoryUsage, 80), 0, 100);

        public static int PingValue => SHS.Clamp(SMMI.BackgroundogSettingManager.GetSettingStable(SMMCB.PingValue, 100), 0, 1000);

        public static string NetworkAdapter => SMMI.BackgroundogSettingManager.GetSetting(SMMCB.NetworkAdapter, string.Empty);

        public static string GraphicAdapter => SMMI.BackgroundogSettingManager.GetSetting(SMMCB.GraphicAdapter, string.Empty);

        public static int GpuUsage => SHS.Clamp(SMMI.BackgroundogSettingManager.GetSettingStable(SMMCB.GpuUsage, 70), 0, 100);

        public static int CpuUsage => SHS.Clamp(SMMI.BackgroundogSettingManager.GetSettingStable(SMMCB.CpuUsage, 70), 0, 100);

        public static bool PerformanceCounter => SMMI.BackgroundogSettingManager.GetSetting(SMMCB.PerformanceCounter, true);

        public static SESET DownloadType => SMMI.BackgroundogSettingManager.GetSetting(SMMCB.DownloadType, SESET.Megabyte);

        public static bool PausePerformance => SMMI.BackgroundogSettingManager.GetSetting(SMMCB.PausePerformance, false);

        public static bool ClosePerformance => SMMI.BackgroundogSettingManager.GetSetting(SMMCB.ClosePerformance, false);

        public static SESET UploadType => SMMI.BackgroundogSettingManager.GetSetting(SMMCB.UploadType, SESET.Kilobyte);

        public static bool SignalRequired => SMMI.BackgroundogSettingManager.GetSetting(SMMCB.SignalRequired, false);

        public static bool AudioRequired => SMMI.BackgroundogSettingManager.GetSetting(SMMCB.AudioRequired, false);

        public static bool PipeRequired => SMMI.BackgroundogSettingManager.GetSetting(SMMCB.PipeRequired, false);

        public static string PingType => SMMI.BackgroundogSettingManager.GetSetting(SMMCB.PingType, "Bing");
    }
}