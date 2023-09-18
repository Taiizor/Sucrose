using LibreHardwareMonitor.Hardware;
using SBSDCS = Sucrose.Backgroundog.Struct.Data.CpuStruct;
using SBSDMS = Sucrose.Backgroundog.Struct.Data.MemoryStruct;
using Timer = System.Threading.Timer;

namespace Sucrose.Backgroundog.Manage
{
    internal static class Internal
    {
        public static bool Exit = true;

        public static bool State = true;

        public static bool Condition = false;

        public static bool Processing = true;

        public static Timer InitializeTimer = null;

        public static SBSDCS CpuData = new()
        {
            Min = 0f,
            Now = 0f,
            Max = 0f
        };

        public static SBSDMS MemoryData = new()
        {
            MemoryUsed = 0f,
            MemoryLoad = 0f,
            MemoryAvailable = 0f,
            VirtualMemoryUsed = 0f,
            VirtualMemoryLoad = 0f,
            VirtualMemoryAvailable = 0f,
        };

        public static Computer Computer = new()
        {
            IsCpuEnabled = true,
            IsGpuEnabled = false,
            IsPsuEnabled = false,
            IsMemoryEnabled = true,
            IsNetworkEnabled = false,
            IsStorageEnabled = false,
            IsBatteryEnabled = true,
            IsControllerEnabled = false,
            IsMotherboardEnabled = false,
        };
    }
}