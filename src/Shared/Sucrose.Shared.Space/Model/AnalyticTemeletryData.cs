using Newtonsoft.Json;

namespace Sucrose.Shared.Space.Model
{
    internal class AnalyticTelemetryData
    {
        [JsonProperty("AppExit", Required = Required.Always)]
        public bool AppExit { get; set; }

        [JsonProperty("IsServer", Required = Required.Always)]
        public bool IsServer { get; set; }

        [JsonProperty("RunStartup", Required = Required.Always)]
        public int RunStartup { get; set; }

        [JsonProperty("AppVisible", Required = Required.Always)]
        public bool AppVisible { get; set; }

        [JsonProperty("UpdateAuto", Required = Required.Always)]
        public bool UpdateAuto { get; set; }

        [JsonProperty("UserName", Required = Required.Always)]
        public string UserName { get; set; }

        [JsonProperty("StoreAdult", Required = Required.Always)]
        public bool StoreAdult { get; set; }

        [JsonProperty("StoreStart", Required = Required.Always)]
        public bool StoreStart { get; set; }

        [JsonProperty("EngineGif", Required = Required.Always)]
        public string EngineGif { get; set; }

        [JsonProperty("EngineUrl", Required = Required.Always)]
        public string EngineUrl { get; set; }

        [JsonProperty("EngineWeb", Required = Required.Always)]
        public string EngineWeb { get; set; }

        [JsonProperty("LibraryMove", Required = Required.Always)]
        public bool LibraryMove { get; set; }

        [JsonProperty("ThemeType", Required = Required.Always)]
        public string ThemeType { get; set; }

        [JsonProperty("TotalMemory", Required = Required.Always)]
        public long TotalMemory { get; set; }

        [JsonProperty("AppVersion", Required = Required.Always)]
        public string AppVersion { get; set; }

        [JsonProperty("CultureCode", Required = Required.Always)]
        public string CultureCode { get; set; }

        [JsonProperty("CultureName", Required = Required.Always)]
        public string CultureName { get; set; }

        [JsonProperty("CyclingActive", Required = Required.Always)]
        public bool CyclingActive { get; set; }

        [JsonProperty("DeviceModel", Required = Required.Always)]
        public string DeviceModel { get; set; }

        [JsonProperty("EngineVideo", Required = Required.Always)]
        public string EngineVideo { get; set; }

        [JsonProperty("IsWorkstation", Required = Required.Always)]
        public bool IsWorkstation { get; set; }

        [JsonProperty("StretchType", Required = Required.Always)]
        public string StretchType { get; set; }

        [JsonProperty("WallpaperLoop", Required = Required.Always)]
        public bool WallpaperLoop { get; set; }

        [JsonProperty("AppFramework", Required = Required.Always)]
        public string AppFramework { get; set; }

        [JsonProperty("DiscordConnect", Required = Required.Always)]
        public bool DiscordConnect { get; set; }

        [JsonProperty("WallpaperVolume", Required = Required.Always)]
        public int WallpaperVolume { get; set; }

        [JsonProperty("EngineYouTube", Required = Required.Always)]
        public string EngineYouTube { get; set; }

        [JsonProperty("CultureDisplay", Required = Required.Always)]
        public string CultureDisplay { get; set; }

        [JsonProperty("GraphicAdapter", Required = Required.Always)]
        public string GraphicAdapter { get; set; }

        [JsonProperty("NetworkAdapter", Required = Required.Always)]
        public string NetworkAdapter { get; set; }

        [JsonProperty("UserIdentifier", Required = Required.Always)]
        public string UserIdentifier { get; set; }

        [JsonProperty("WallpaperShuffle", Required = Required.Always)]
        public bool WallpaperShuffle { get; set; }

        [JsonProperty("AdvertisingActive", Required = Required.Always)]
        public bool AdvertisingActive { get; set; }

        [JsonProperty("EngineInputType", Required = Required.Always)]
        public string EngineInputType { get; set; }

        [JsonProperty("GraphicAdapters", Required = Required.Always)]
        public string GraphicAdapters { get; set; }

        [JsonProperty("LibraryLocation", Required = Required.Always)]
        public string LibraryLocation { get; set; }

        [JsonProperty("NetworkAdapter", Required = Required.Always)]
        public string NetworkAdapters { get; set; }

        [JsonProperty("StoreServerType", Required = Required.Always)]
        public string StoreServerType { get; set; }

        [JsonProperty("UserIdentifying", Required = Required.Always)]
        public string UserIdentifying { get; set; }

        [JsonProperty("DeviceProcessors", Required = Required.Always)]
        public string DeviceProcessors { get; set; }

        [JsonProperty("EngineScreenType", Required = Required.Always)]
        public string EngineScreenType { get; set; }

        [JsonProperty("EngineInputDesktop", Required = Required.Always)]
        public bool EngineInputDesktop { get; set; }

        [JsonProperty("UpdateModuleType", Required = Required.Always)]
        public string UpdateModuleType { get; set; }

        [JsonProperty("UpdateServerType", Required = Required.Always)]
        public string UpdateServerType { get; set; }

        [JsonProperty("EngineApplication", Required = Required.Always)]
        public string EngineApplication { get; set; }

        [JsonProperty("UpdateChannelType", Required = Required.Always)]
        public string UpdateChannelType { get; set; }

        [JsonProperty("CyclingTransitionTime", Required = Required.Always)]
        public int CyclingTransitionTime { get; set; }

        [JsonProperty("UpdateExtensionType", Required = Required.Always)]
        public string UpdateExtensionType { get; set; }
    }

    internal class AnalyticsData(bool Report, bool Statistics, string InputModule, bool LibraryStart, bool StorePreview, bool VolumeActive, int DeveloperPort, int NumberOfCores, int StoreDuration, string Architecture, bool DeveloperMode, bool DonateVisible, bool LibraryDelete, string Manufacturer, bool VolumeDesktop, string Communication, string DisplayScreen, bool LibraryConfirm, bool LibraryPreview, int StorePagination, string CpuPerformance, string GpuPerformance, string TransitionType, int AdvertisingDelay, string BackgroundImage, string OperatingSystem, int BackgroundOpacity, int LibraryPagination, string FocusPerformance, string PausePerformance, string SaverPerformance, int NumberOfProcessors, string BackgroundStretch, string MemoryPerformance, bool PerformanceCounter, string RemotePerformance, string BatteryPerformance, string NetworkPerformance, string VirtualPerformance, string ProcessArchitecture, string OperatingSystemBuild, string FullscreenPerformance, string ProcessorArchitecture, string OperatingSystemArchitecture)
    {
        public bool Report { get; set; } = Report;

        public bool Statistics { get; set; } = Statistics;

        public string InputModule { get; set; } = InputModule;

        public bool LibraryStart { get; set; } = LibraryStart;

        public bool StorePreview { get; set; } = StorePreview;

        public bool VolumeActive { get; set; } = VolumeActive;

        public int DeveloperPort { get; set; } = DeveloperPort;

        public int NumberOfCores { get; set; } = NumberOfCores;

        public int StoreDuration { get; set; } = StoreDuration;

        public string Architecture { get; set; } = Architecture;

        public bool DeveloperMode { get; set; } = DeveloperMode;

        public bool DonateVisible { get; set; } = DonateVisible;

        public bool LibraryDelete { get; set; } = LibraryDelete;

        public string Manufacturer { get; set; } = Manufacturer;

        public bool VolumeDesktop { get; set; } = VolumeDesktop;

        public string Communication { get; set; } = Communication;

        public string DisplayScreen { get; set; } = DisplayScreen;

        public bool LibraryConfirm { get; set; } = LibraryConfirm;

        public bool LibraryPreview { get; set; } = LibraryPreview;

        public int StorePagination { get; set; } = StorePagination;

        public string CpuPerformance { get; set; } = CpuPerformance;

        public string GpuPerformance { get; set; } = GpuPerformance;

        public string TransitionType { get; set; } = TransitionType;

        public int AdvertisingDelay { get; set; } = AdvertisingDelay;

        public string BackgroundImage { get; set; } = BackgroundImage;

        public string OperatingSystem { get; set; } = OperatingSystem;

        public int BackgroundOpacity { get; set; } = BackgroundOpacity;

        public int LibraryPagination { get; set; } = LibraryPagination;

        public string FocusPerformance { get; set; } = FocusPerformance;

        public string PausePerformance { get; set; } = PausePerformance;

        public string SaverPerformance { get; set; } = SaverPerformance;

        public int NumberOfProcessors { get; set; } = NumberOfProcessors;

        public string BackgroundStretch { get; set; } = BackgroundStretch;

        public string MemoryPerformance { get; set; } = MemoryPerformance;

        public bool PerformanceCounter { get; set; } = PerformanceCounter;

        public string RemotePerformance { get; set; } = RemotePerformance;

        public string BatteryPerformance { get; set; } = BatteryPerformance;

        public string NetworkPerformance { get; set; } = NetworkPerformance;

        public string VirtualPerformance { get; set; } = VirtualPerformance;

        public string ProcessArchitecture { get; set; } = ProcessArchitecture;

        public string OperatingSystemBuild { get; set; } = OperatingSystemBuild;

        public string FullscreenPerformance { get; set; } = FullscreenPerformance;

        public string ProcessorArchitecture { get; set; } = ProcessorArchitecture;

        public string OperatingSystemArchitecture { get; set; } = OperatingSystemArchitecture;
    }
}