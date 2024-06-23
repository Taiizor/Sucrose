using Newtonsoft.Json.Linq;
using NPSMLib;
using System.Runtime.InteropServices;
using SSPPSS = Skylark.Struct.Ping.PingSendStruct;
using SSSSS = Skylark.Struct.Storage.StorageStruct;

namespace Sucrose.Backgroundog.Struct.Data
{
    /// <summary>
    /// 
    /// </summary>
    [StructLayout(LayoutKind.Sequential)]
    public struct CpuStruct
    {
        /// <summary>
        /// 
        /// </summary>
        public int? Core;
        /// <summary>
        /// 
        /// </summary>
        public float? Min;
        /// <summary>
        /// 
        /// </summary>
        public float? Now;
        /// <summary>
        /// 
        /// </summary>
        public float? Max;
        /// <summary>
        /// 
        /// </summary>
        public bool State;
        /// <summary>
        /// 
        /// </summary>
        public int? Thread;
        /// <summary>
        /// 
        /// </summary>
        public string Name;
        /// <summary>
        /// 
        /// </summary>
        public string FullName;
    }

    /// <summary>
    /// 
    /// </summary>
    [StructLayout(LayoutKind.Sequential)]
    public struct BiosStruct
    {
        /// <summary>
        /// 
        /// </summary>
        public bool State;
        /// <summary>
        /// 
        /// </summary>
        public string Name;
        /// <summary>
        /// 
        /// </summary>
        public string Caption;
        /// <summary>
        /// 
        /// </summary>
        public string Version;
        /// <summary>
        /// 
        /// </summary>
        public string Description;
        /// <summary>
        /// 
        /// </summary>
        public string ReleaseDate;
        /// <summary>
        /// 
        /// </summary>
        public string Manufacturer;
        /// <summary>
        /// 
        /// </summary>
        public string SerialNumber;
        /// <summary>
        /// 
        /// </summary>
        public string CurrentLanguage;
    }

    /// <summary>
    /// 
    /// </summary>
    [StructLayout(LayoutKind.Sequential)]
    public struct DateStruct
    {
        /// <summary>
        /// 
        /// </summary>
        public int Day;
        /// <summary>
        /// 
        /// </summary>
        public int Hour;
        /// <summary>
        /// 
        /// </summary>
        public int Year;
        /// <summary>
        /// 
        /// </summary>
        public int Month;
        /// <summary>
        /// 
        /// </summary>
        public int Minute;
        /// <summary>
        /// 
        /// </summary>
        public int Second;
        /// <summary>
        /// 
        /// </summary>
        public bool State;
        /// <summary>
        /// 
        /// </summary>
        public int Millisecond;
    }

    /// <summary>
    /// 
    /// </summary>
    [StructLayout(LayoutKind.Sequential)]
    public struct AudioStruct
    {
        /// <summary>
        /// 
        /// </summary>
        private uint? PID;
        /// <summary>
        /// 
        /// </summary>
        public bool State;
        /// <summary>
        /// 
        /// </summary>
        public string Title;
        /// <summary>
        /// 
        /// </summary>
        private IntPtr? Hwnd;
        /// <summary>
        /// 
        /// </summary>
        public string Artist;
        /// <summary>
        /// 
        /// </summary>
        public double[] Data;
        /// <summary>
        /// 
        /// </summary>
        public string Subtitle;
        /// <summary>
        /// 
        /// </summary>
        public TimeSpan EndTime;
        /// <summary>
        /// 
        /// </summary>
        public uint TrackNumber;
        /// <summary>
        /// 
        /// </summary>
        public string AlbumTitle;
        /// <summary>
        /// 
        /// </summary>
        public TimeSpan Position;
        /// <summary>
        /// 
        /// </summary>
        public string AlbumArtist;
        /// <summary>
        /// 
        /// </summary>
        public string SourceAppId;
        /// <summary>
        /// 
        /// </summary>
        public TimeSpan StartTime;
        /// <summary>
        /// 
        /// </summary>
        public double PlaybackRate;
        /// <summary>
        /// 
        /// </summary>
        public bool ShuffleEnabled;
        /// <summary>
        /// 
        /// </summary>
        public uint AlbumTrackCount;
        /// <summary>
        /// 
        /// </summary>
        public TimeSpan MinSeekTime;
        /// <summary>
        /// 
        /// </summary>
        public TimeSpan MaxSeekTime;
        /// <summary>
        /// 
        /// </summary>
        private string SourceDeviceId;
        /// <summary>
        /// 
        /// </summary>
        private string RenderDeviceId;
        /// <summary>
        /// 
        /// </summary>
        public string ThumbnailString;
        /// <summary>
        /// 
        /// </summary>
        public MediaPlaybackMode MediaType;
        /// <summary>
        /// 
        /// </summary>
        public DateTime LastPlayingFileTime;
        /// <summary>
        /// 
        /// </summary>
        public DateTime PositionSetFileTime;
        /// <summary>
        /// 
        /// </summary>
        public MediaPlaybackProps PropsValid;
        /// <summary>
        /// 
        /// </summary>
        public MediaPlaybackMode PlaybackMode;
        /// <summary>
        /// 
        /// </summary>
        public MediaPlaybackState PlaybackState;
        /// <summary>
        /// 
        /// </summary>
        public MediaPlaybackRepeatMode RepeatMode;
        /// <summary>
        /// 
        /// </summary>
        private MediaPlaybackCapabilities PlaybackCaps;
    }

    /// <summary>
    /// 
    /// </summary>
    [StructLayout(LayoutKind.Sequential)]
    public struct MemoryStruct
    {
        /// <summary>
        /// 
        /// </summary>
        public bool State;
        /// <summary>
        /// 
        /// </summary>
        public string Name;
        /// <summary>
        /// 
        /// </summary>
        public float? MemoryUsed;
        /// <summary>
        /// 
        /// </summary>
        public float? MemoryLoad;
        /// <summary>
        /// 
        /// </summary>
        public float? MemoryAvailable;
        /// <summary>
        /// 
        /// </summary>
        public float? VirtualMemoryUsed;
        /// <summary>
        /// 
        /// </summary>
        public float? VirtualMemoryLoad;
        /// <summary>
        /// 
        /// </summary>
        public float? VirtualMemoryAvailable;
    }

    /// <summary>
    /// 
    /// </summary>
    [StructLayout(LayoutKind.Sequential)]
    public struct BatteryStruct
    {
        /// <summary>
        /// 
        /// </summary>
        public bool State;
        /// <summary>
        /// 
        /// </summary>
        public string Name;
        /// <summary>
        /// 
        /// </summary>
        public float? Voltage;
        /// <summary>
        /// 
        /// </summary>
        public bool SavingMode;
        /// <summary>
        /// 
        /// </summary>
        public int FullLifetime;
        /// <summary>
        /// 
        /// </summary>
        public float? ChargeRate;
        /// <summary>
        /// 
        /// </summary>
        public int LifeRemaining;
        /// <summary>
        /// 
        /// </summary>
        public float LifePercent;
        /// <summary>
        /// 
        /// </summary>
        public string SaverStatus;
        /// <summary>
        /// 
        /// </summary>
        public float? ChargeLevel;
        /// <summary>
        /// 
        /// </summary>
        public string ACPowerStatus;
        /// <summary>
        /// 
        /// </summary>
        public float? ChargeCurrent;
        /// <summary>
        /// 
        /// </summary>
        public float? DischargeRate;
        /// <summary>
        /// 
        /// </summary>
        public float? DischargeLevel;
        /// <summary>
        /// 
        /// </summary>
        public float? DischargeCurrent;
        /// <summary>
        /// 
        /// </summary>
        public float? DegradationLevel;
        /// <summary>
        /// 
        /// </summary>
        public float? DesignedCapacity;
        /// <summary>
        /// 
        /// </summary>
        public float? RemainingCapacity;
        /// <summary>
        /// 
        /// </summary>
        public float? FullChargedCapacity;
        /// <summary>
        /// 
        /// </summary>
        public float? ChargeDischargeRate;
        /// <summary>
        /// 
        /// </summary>
        public float? ChargeDischargeCurrent;
        /// <summary>
        /// 
        /// </summary>
        public float? RemainingTimeEstimated;
        /// <summary>
        /// 
        /// </summary>
        public PowerLineStatus PowerLineStatus;
        /// <summary>
        /// 
        /// </summary>
        public BatteryChargeStatus ChargeStatus;
    }

    /// <summary>
    /// 
    /// </summary>
    [StructLayout(LayoutKind.Sequential)]
    public struct GraphicStruct
    {
        /// <summary>
        /// 
        /// </summary>
        public bool State;
        /// <summary>
        /// 
        /// </summary>
        public JArray Amd;
        /// <summary>
        /// 
        /// </summary>
        public JArray Intel;
        /// <summary>
        /// 
        /// </summary>
        public JArray Nvidia;
    }

    /// <summary>
    /// 
    /// </summary>
    [StructLayout(LayoutKind.Sequential)]
    public struct NetworkStruct
    {
        /// <summary>
        /// 
        /// </summary>
        public long Ping;
        /// <summary>
        /// 
        /// </summary>
        public bool State;
        /// <summary>
        /// 
        /// </summary>
        public string Name;
        /// <summary>
        /// 
        /// </summary>
        public string Host;
        /// <summary>
        /// 
        /// </summary>
        public float Upload;
        /// <summary>
        /// 
        /// </summary>
        public float Download;
        /// <summary>
        /// 
        /// </summary>
        public SSPPSS PingData;
        /// <summary>
        /// 
        /// </summary>
        public SSSSS UploadData;
        /// <summary>
        /// 
        /// </summary>
        public SSSSS DownloadData;
        /// <summary>
        /// 
        /// </summary>
        public string PingAddress;
        /// <summary>
        /// 
        /// </summary>
        public string FormatUploadData;
        /// <summary>
        /// 
        /// </summary>
        public string FormatDownloadData;
    }

    /// <summary>
    /// 
    /// </summary>
    [StructLayout(LayoutKind.Sequential)]
    public struct MotherboardStruct
    {
        /// <summary>
        /// 
        /// </summary>
        public bool State;
        /// <summary>
        /// 
        /// </summary>
        public string Name;
        /// <summary>
        /// 
        /// </summary>
        public string Product;
        /// <summary>
        /// 
        /// </summary>
        public string Version;
        /// <summary>
        /// 
        /// </summary>
        public string Manufacturer;
    }
}