using LibreHardwareMonitor.Hardware;
using System.Diagnostics;
using SBHI = Sucrose.Backgroundog.Helper.Initialize;
using SBSDBSS = Sucrose.Backgroundog.Struct.Data.BiosStruct;
using SBSDBYS = Sucrose.Backgroundog.Struct.Data.BatteryStruct;
using SBSDCS = Sucrose.Backgroundog.Struct.Data.CpuStruct;
using SBSDMDS = Sucrose.Backgroundog.Struct.Data.MotherboardStruct;
using SBSDMYS = Sucrose.Backgroundog.Struct.Data.MemoryStruct;
using SBSDNS = Sucrose.Backgroundog.Struct.Data.NetworkStruct;
using Timer = System.Threading.Timer;

namespace Sucrose.Backgroundog.Manage
{
    internal static class Internal
    {
        public static bool Exit = true;

        public static bool Condition = false;

        public static bool Processing = true;

        public static SBHI Initialize = new();

        public static bool CpuManagement = true;

        public static bool BiosManagement = true;

        public static Timer InitializeTimer = null;

        public static PerformanceCounter UploadCounter = null;

        public static PerformanceCounter DownloadCounter = null;

        public static SBSDCS CpuData = new()
        {
            Min = 0f,
            Now = 0f,
            Max = 0f,
            Core = 0,
            Thread = 0,
            Name = string.Empty,
            Fullname = string.Empty
        };

        public static SBSDBSS BiosData = new()
        {
            Name = string.Empty,
            Caption = string.Empty,
            Version = string.Empty,
            Description = string.Empty,
            ReleaseDate = string.Empty,
            Manufacturer = string.Empty,
            SerialNumber = string.Empty,
            CurrentLanguage = string.Empty
        };

        public static SBSDMYS MemoryData = new()
        {
            MemoryUsed = 0f,
            MemoryLoad = 0f,
            Name = string.Empty,
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
            IsBatteryEnabled = true,
            IsNetworkEnabled = false,
            IsStorageEnabled = false,
            IsControllerEnabled = false,
            IsMotherboardEnabled = true,
        };

        public static SBSDNS NetworkData = new()
        {
            Upload = 0f,
            Download = 0f,
            UploadData = new(),
            Name = string.Empty,
            DownloadData = new(),
            FormatUploadData = string.Empty,
            FormatDownloadData = string.Empty
        };

        public static SBSDBYS BatteryData = new()
        {
            Voltage = 0f,
            ChargeRate = 0f,
            ChargeLevel = 0f,
            ChargeCurrent = 0f,
            DischargeRate = 0f,
            DischargeLevel = 0f,
            Name = string.Empty,
            DischargeCurrent = 0f,
            DegradationLevel = 0f,
            DesignedCapacity = 0f,
            RemainingCapacity = 0f,
            FullChargedCapacity = 0f,
            ChargeDischargeRate = 0f,
            ChargeDischargeCurrent = 0f,
            RemainingTimeEstimated = 0f
        };

        public static SBSDMDS MotherboardData = new()
        {
            Name = string.Empty
        };
    }
}