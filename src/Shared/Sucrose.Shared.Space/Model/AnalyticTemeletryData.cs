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

        [JsonProperty("StoreAdult", Required = Required.Always)]
        public bool StoreAdult { get; set; }

        [JsonProperty("StoreStart", Required = Required.Always)]
        public bool StoreStart { get; set; }

        [JsonProperty("UpdateAuto", Required = Required.Always)]
        public bool UpdateAuto { get; set; }

        [JsonProperty("UserName", Required = Required.Always)]
        public string UserName { get; set; }

        [JsonProperty("EngineGif", Required = Required.Always)]
        public string EngineGif { get; set; }

        [JsonProperty("EngineUrl", Required = Required.Always)]
        public string EngineUrl { get; set; }

        [JsonProperty("EngineWeb", Required = Required.Always)]
        public string EngineWeb { get; set; }

        [JsonProperty("IsException", Required = Required.Always)]
        public bool IsException { get; set; }

        [JsonProperty("IsTelemetry", Required = Required.Always)]
        public bool IsTelemetry { get; set; }

        [JsonProperty("LibraryMove", Required = Required.Always)]
        public bool LibraryMove { get; set; }

        [JsonProperty("ThemeType", Required = Required.Always)]
        public string ThemeType { get; set; }

        [JsonProperty("TotalMemory", Required = Required.Always)]
        public long TotalMemory { get; set; }

        [JsonProperty("AppVersion", Required = Required.Always)]
        public string AppVersion { get; set; }

        [JsonProperty("DeveloperPort", Required = Required.Always)]
        public int DeveloperPort { get; set; }

        [JsonProperty("LibraryStart", Required = Required.Always)]
        public bool LibraryStart { get; set; }

        [JsonProperty("NumberOfCores", Required = Required.Always)]
        public int NumberOfCores { get; set; }

        [JsonProperty("StoreDuration", Required = Required.Always)]
        public int StoreDuration { get; set; }

        [JsonProperty("StorePreview", Required = Required.Always)]
        public bool StorePreview { get; set; }

        [JsonProperty("VolumeSilent", Required = Required.Always)]
        public bool VolumeSilent { get; set; }

        [JsonProperty("CultureCode", Required = Required.Always)]
        public string CultureCode { get; set; }

        [JsonProperty("CultureName", Required = Required.Always)]
        public string CultureName { get; set; }

        [JsonProperty("CyclingActive", Required = Required.Always)]
        public bool CyclingActive { get; set; }

        [JsonProperty("DeveloperMode", Required = Required.Always)]
        public bool DeveloperMode { get; set; }

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

        [JsonProperty("LibraryPreview", Required = Required.Always)]
        public bool LibraryPreview { get; set; }

        [JsonProperty("StorePagination", Required = Required.Always)]
        public int StorePagination { get; set; }

        [JsonProperty("WallpaperVolume", Required = Required.Always)]
        public int WallpaperVolume { get; set; }

        [JsonProperty("AdvertisingDelay", Required = Required.Always)]
        public int AdvertisingDelay { get; set; }

        [JsonProperty("EngineYouTube", Required = Required.Always)]
        public string EngineYouTube { get; set; }

        [JsonProperty("BackgroundOpacity", Required = Required.Always)]
        public int BackgroundOpacity { get; set; }

        [JsonProperty("CpuPerformance", Required = Required.Always)]
        public string CpuPerformance { get; set; }

        [JsonProperty("CultureDisplay", Required = Required.Always)]
        public string CultureDisplay { get; set; }

        [JsonProperty("GpuPerformance", Required = Required.Always)]
        public string GpuPerformance { get; set; }

        [JsonProperty("GraphicAdapter", Required = Required.Always)]
        public string GraphicAdapter { get; set; }

        [JsonProperty("LibraryPagination", Required = Required.Always)]
        public int LibraryPagination { get; set; }

        [JsonProperty("NetworkAdapter", Required = Required.Always)]
        public string NetworkAdapter { get; set; }

        [JsonProperty("UserIdentifier", Required = Required.Always)]
        public string UserIdentifier { get; set; }

        [JsonProperty("WallpaperShuffle", Required = Required.Always)]
        public bool WallpaperShuffle { get; set; }

        [JsonProperty("AdvertisingActive", Required = Required.Always)]
        public bool AdvertisingActive { get; set; }

        [JsonProperty("AppArchitecture", Required = Required.Always)]
        public string AppArchitecture { get; set; }

        [JsonProperty("BackgroundImage", Required = Required.Always)]
        public string BackgroundImage { get; set; }

        [JsonProperty("EngineInputType", Required = Required.Always)]
        public string EngineInputType { get; set; }

        [JsonProperty("GraphicAdapters", Required = Required.Always)]
        public string GraphicAdapters { get; set; }

        [JsonProperty("InputModuleType", Required = Required.Always)]
        public string InputModuleType { get; set; }

        [JsonProperty("LibraryLocation", Required = Required.Always)]
        public string LibraryLocation { get; set; }

        [JsonProperty("NetworkAdapter", Required = Required.Always)]
        public string NetworkAdapters { get; set; }

        [JsonProperty("NumberOfProcessors", Required = Required.Always)]
        public int NumberOfProcessors { get; set; }

        [JsonProperty("OperatingSystem", Required = Required.Always)]
        public string OperatingSystem { get; set; }

        [JsonProperty("StoreServerType", Required = Required.Always)]
        public string StoreServerType { get; set; }

        [JsonProperty("UserIdentifying", Required = Required.Always)]
        public string UserIdentifying { get; set; }

        [JsonProperty("DeviceProcessors", Required = Required.Always)]
        public string DeviceProcessors { get; set; }

        [JsonProperty("EngineScreenType", Required = Required.Always)]
        public string EngineScreenType { get; set; }
		
        [JsonProperty("FocusPerformance", Required = Required.Always)]
        public string FocusPerformance { get; set; }

        [JsonProperty("EngineInputDesktop", Required = Required.Always)]
        public bool EngineInputDesktop { get; set; }

        [JsonProperty("PausePerformance", Required = Required.Always)]
        public string PausePerformance { get; set; }

        [JsonProperty("PerformanceCounter", Required = Required.Always)]
        public bool PerformanceCounter { get; set; }

        [JsonProperty("SaverPerformance", Required = Required.Always)]
        public string SaverPerformance { get; set; }

        [JsonProperty("UpdateModuleType", Required = Required.Always)]
        public string UpdateModuleType { get; set; }

        [JsonProperty("UpdateServerType", Required = Required.Always)]
        public string UpdateServerType { get; set; }

        [JsonProperty("BackgroundStretch", Required = Required.Always)]
        public string BackgroundStretch { get; set; }

        [JsonProperty("CommunicationType", Required = Required.Always)]
        public string CommunicationType { get; set; }

        [JsonProperty("DisplayScreenType", Required = Required.Always)]
        public string DisplayScreenType { get; set; }

        [JsonProperty("EngineApplication", Required = Required.Always)]
        public string EngineApplication { get; set; }

        [JsonProperty("EngineVolumeDesktop", Required = Required.Always)]
        public bool EngineVolumeDesktop { get; set; }

        [JsonProperty("ManufacturerBrand", Required = Required.Always)]
        public string ManufacturerBrand { get; set; }

        [JsonProperty("MemoryPerformance", Required = Required.Always)]
        public string MemoryPerformance { get; set; }

        [JsonProperty("RemotePerformance", Required = Required.Always)]
        public string RemotePerformance { get; set; }

        [JsonProperty("UpdateChannelType", Required = Required.Always)]
        public string UpdateChannelType { get; set; }

        [JsonProperty("BatteryPerformance", Required = Required.Always)]
        public string BatteryPerformance { get; set; }

        [JsonProperty("CyclingTransitionTime", Required = Required.Always)]
        public int CyclingTransitionTime { get; set; }

        [JsonProperty("LibraryDeleteConfirm", Required = Required.Always)]
        public bool LibraryDeleteConfirm { get; set; }

        [JsonProperty("LibraryDeleteCorrupt", Required = Required.Always)]
        public bool LibraryDeleteCorrupt { get; set; }

        [JsonProperty("NetworkPerformance", Required = Required.Always)]
        public string NetworkPerformance { get; set; }

        [JsonProperty("VirtualPerformance", Required = Required.Always)]
        public string VirtualPerformance { get; set; }

        [JsonProperty("ProcessArchitecture", Required = Required.Always)]
        public string ProcessArchitecture { get; set; }

        [JsonProperty("UpdateExtensionType", Required = Required.Always)]
        public string UpdateExtensionType { get; set; }

        [JsonProperty("AdvertisingMenuVisible", Required = Required.Always)]
        public bool AdvertisingMenuVisible { get; set; }

        [JsonProperty("OperatingSystemBuild", Required = Required.Always)]
        public string OperatingSystemBuild { get; set; }

        [JsonProperty("VolumeSilentSensitivity", Required = Required.Always)]
        public int VolumeSilentSensitivity { get; set; }

        [JsonProperty("CyclingTransitionType", Required = Required.Always)]
        public string CyclingTransitionType { get; set; }

        [JsonProperty("FullscreenPerformance", Required = Required.Always)]
        public string FullscreenPerformance { get; set; }

        [JsonProperty("ProcessorArchitecture", Required = Required.Always)]
        public string ProcessorArchitecture { get; set; }

        [JsonProperty("OperatingSystemArchitecture", Required = Required.Always)]
        public string OperatingSystemArchitecture { get; set; }
    }
}