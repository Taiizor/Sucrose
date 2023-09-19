using NPSMLib;
using System.Runtime.InteropServices;
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
        public int? Thread;
        /// <summary>
        /// 
        /// </summary>
        public string Name;
        /// <summary>
        /// 
        /// </summary>
        public string Fullname;
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
        public uint? PID;
        /// <summary>
        /// 
        /// </summary>
        public IntPtr? Hwnd;
        /// <summary>
        /// 
        /// </summary>
        public string Title;
        /// <summary>
        /// 
        /// </summary>
        public string Artist;
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
        public string AlbumTitle;
        /// <summary>
        /// 
        /// </summary>
        public TimeSpan Position;
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
        public TimeSpan MinSeekTime;
        /// <summary>
        /// 
        /// </summary>
        public TimeSpan MaxSeekTime;
        /// <summary>
        /// 
        /// </summary>
        public string SourceDeviceId;
        /// <summary>
        /// 
        /// </summary>
        public string RenderDeviceId;
        /// <summary>
        /// 
        /// </summary>
        public string ThumbnailString;
        /// <summary>
        /// 
        /// </summary>
        public string ThumbnailAddress;
        /// <summary>
        /// 
        /// </summary>
        public MediaPlaybackMode MediaType;
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
        public MediaPlaybackCapabilities PlaybackCaps;
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
        public string Name;
        /// <summary>
        /// 
        /// </summary>
        public float? Voltage;
        /// <summary>
        /// 
        /// </summary>
        public float? ChargeRate;
        /// <summary>
        /// 
        /// </summary>
        public float? ChargeLevel;
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
        public string Name;
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
        public SSSSS UploadData;
        /// <summary>
        /// 
        /// </summary>
        public SSSSS DownloadData;
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
        public string Name;
    }
}