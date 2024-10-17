using Newtonsoft.Json;

namespace Sucrose.Shared.Space.Model
{
    internal class DownloadTelemetryData
    {
        [JsonProperty("AppVersion", Required = Required.Always)]
        public string AppVersion { get; set; }

        [JsonProperty("WallpaperTitle", Required = Required.Always)]
        public string WallpaperTitle { get; set; }

        [JsonProperty("WallpaperVersion", Required = Required.Always)]
        public string WallpaperVersion { get; set; }

        [JsonProperty("WallpaperLocation", Required = Required.Always)]
        public string WallpaperLocation { get; set; }

        [JsonProperty("WallpaperAppVersion", Required = Required.Always)]
        public string WallpaperAppVersion { get; set; }
    }
}