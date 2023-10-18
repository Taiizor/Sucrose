using LibreHardwareMonitor.Hardware;
using NPSMLib;
using System.Diagnostics;
using SBHI = Sucrose.Backgroundog.Helper.Initialize;
using SBSDAS = Sucrose.Backgroundog.Struct.Data.AudioStruct;
using SBSDBSS = Sucrose.Backgroundog.Struct.Data.BiosStruct;
using SBSDBYS = Sucrose.Backgroundog.Struct.Data.BatteryStruct;
using SBSDCS = Sucrose.Backgroundog.Struct.Data.CpuStruct;
using SBSDDS = Sucrose.Backgroundog.Struct.Data.DateStruct;
using SBSDGS = Sucrose.Backgroundog.Struct.Data.GraphicStruct;
using SBSDMDS = Sucrose.Backgroundog.Struct.Data.MotherboardStruct;
using SBSDMYS = Sucrose.Backgroundog.Struct.Data.MemoryStruct;
using SBSDNS = Sucrose.Backgroundog.Struct.Data.NetworkStruct;
using SSDECPT = Sucrose.Shared.Dependency.Enum.CategoryPerformanceType;
using SSDENPT = Sucrose.Shared.Dependency.Enum.NetworkPerformanceType;
using SSDEPT = Sucrose.Shared.Dependency.Enum.PerformanceType;
using Timer = System.Threading.Timer;

namespace Sucrose.Backgroundog.Manage
{
    internal static class Internal
    {
        public static bool Exit = true;

        public static int AppTime = 1000;

        public static Process App = null;

        public static Process Live = null;

        public static bool Condition = false;

        public static bool Processing = true;

        public static bool Fullscreen = false;

        public static SBHI Initialize = new();

        public static bool Virtuality = false;

        public static int InitializeTime = 250;

        public static bool FocusDesktop = false;

        public static bool CpuManagement = true;

        public static bool BiosManagement = true;

        public static bool PingManagement = true;

        public static bool RemoteDesktop = false;

        public static bool AudioManagement = true;

        public static bool FocusManagement = true;

        public static bool SignalManagement = true;

        public static bool RemoteManagement = true;

        public static Timer InitializeTimer = null;

        public static bool SessionManagement = true;

        public static bool GraphicManagement = true;

        public static bool NetworkManagement = true;

        public static bool BatteryManagement = true;

        public static bool NetworkManagement2 = true;

        public static bool ComputerManagement = true;

        public static bool EqualizerManagement = true;

        public static bool FullscreenManagement = true;

        public static bool VirtualityManagement = true;

        public static bool MotherboardManagement = true;

        public static readonly object LockObject = new();

        public static SSDEPT Performance = SSDEPT.Resume;

        public static NowPlayingSession PlayingSession = null;

        public static PerformanceCounter UploadCounter = null;

        public static SSDENPT NetworkPerformance = SSDENPT.Not;

        public static SSDECPT CategoryPerformance = SSDECPT.Not;

        public static MediaPlaybackDataSource DataSource = null;

        public static PerformanceCounter DownloadCounter = null;

        public static int SpecificationTime = InitializeTime * 8;

        public static int SpecificationLessTime = InitializeTime * 4;

        public static NowPlayingSessionManager SessionManager = null;

        public static string[] NetworkInterfaces = Array.Empty<string>();

        public static SBSDCS CpuData = new()
        {
            Min = 0f,
            Now = 0f,
            Max = 0f,
            Core = 0,
            Thread = 0,
            State = false,
            Name = string.Empty,
            Fullname = string.Empty
        };

        public static SBSDBSS BiosData = new()
        {
            State = false,
            Name = string.Empty,
            Caption = string.Empty,
            Version = string.Empty,
            Description = string.Empty,
            ReleaseDate = string.Empty,
            Manufacturer = string.Empty,
            SerialNumber = string.Empty,
            CurrentLanguage = string.Empty
        };

        public static SBSDDS DateData = new()
        {
            Day = 0,
            Hour = 0,
            Year = 0,
            Month = 0,
            Minute = 0,
            Second = 0,
            State = false,
            Millisecond = 0
        };

        public static SBSDAS AudioData = new()
        {
            //PID = 0,
            State = false,
            TrackNumber = 0,
            PlaybackRate = 0d,
            //Hwnd = IntPtr.Zero,
            AlbumTrackCount = 0,
            Title = string.Empty,
            Artist = string.Empty,
            ShuffleEnabled = false,
            Subtitle = string.Empty,
            EndTime = TimeSpan.Zero,
            Position = TimeSpan.Zero,
            AlbumTitle = string.Empty,
            StartTime = TimeSpan.Zero,
            AlbumArtist = string.Empty,
            SourceAppId = string.Empty,
            MinSeekTime = TimeSpan.Zero,
            MaxSeekTime = TimeSpan.Zero,
            LastPlayingFileTime = new(),
            PositionSetFileTime = new(),
            //SourceDeviceId = string.Empty,
            //RenderDeviceId = string.Empty,
            ThumbnailString = string.Empty,
            ThumbnailAddress = string.Empty,
            MediaType = MediaPlaybackMode.Unknown,
            PlaybackMode = MediaPlaybackMode.Unknown,
            PlaybackState = MediaPlaybackState.Unknown,
            PropsValid = MediaPlaybackProps.Capabilities,
            RepeatMode = MediaPlaybackRepeatMode.Unknown,
            //PlaybackCaps = MediaPlaybackCapabilities.None
        };

        public static SBSDMYS MemoryData = new()
        {
            State = false,
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
            IsGpuEnabled = true,
            IsPsuEnabled = false,
            IsMemoryEnabled = true,
            IsBatteryEnabled = true,
            IsNetworkEnabled = false,
            IsStorageEnabled = false,
            IsControllerEnabled = false,
            IsMotherboardEnabled = true,
        };

        public static SBSDGS GraphicData = new()
        {
            Amd = new(),
            State = false,
            Intel = new(),
            Nvidia = new()
        };

        public static SBSDNS NetworkData = new()
        {
            Ping = 0,
            Upload = 0f,
            State = false,
            Download = 0f,
            PingData = new(),
            UploadData = new(),
            Name = string.Empty,
            DownloadData = new(),
            FormatUploadData = string.Empty,
            FormatDownloadData = string.Empty
        };

        public static SBSDBYS BatteryData = new()
        {
            Voltage = 0f,
            State = false,
            ChargeRate = 0f,
            ChargeLevel = 0f,
            LifePercent = 0f,
            FullLifetime = 0,
            LifeRemaining = 0,
            ChargeCurrent = 0f,
            DischargeRate = 0f,
            SavingMode = false,
            DischargeLevel = 0f,
            Name = string.Empty,
            DischargeCurrent = 0f,
            DegradationLevel = 0f,
            DesignedCapacity = 0f,
            RemainingCapacity = 0f,
            FullChargedCapacity = 0f,
            ChargeDischargeRate = 0f,
            ChargeDischargeCurrent = 0f,
            RemainingTimeEstimated = 0f,
            ACPowerStatus = string.Empty,
            PowerLineStatus = PowerLineStatus.Unknown,
            ChargeStatus = BatteryChargeStatus.Unknown
        };

        public static SBSDMDS MotherboardData = new()
        {
            State = false,
            Name = string.Empty,
            Product = string.Empty,
            Version = string.Empty,
            Manufacturer = string.Empty
        };
    }
}