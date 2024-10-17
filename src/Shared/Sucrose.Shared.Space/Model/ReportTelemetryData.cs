using Newtonsoft.Json;

namespace Sucrose.Shared.Space.Model
{
    internal class ReportTelemetryData
    {
        [JsonProperty("AppVersion", Required = Required.Always)]
        public string AppVersion { get; set; }

        [JsonProperty("ContactEmail", Required = Required.Always)]
        public string ContactEmail { get; set; }

        [JsonProperty("WallpaperTitle", Required = Required.Always)]
        public string WallpaperTitle { get; set; }

        [JsonProperty("RelatedCategory", Required = Required.Always)]
        public string RelatedCategory { get; set; }

        [JsonProperty("WallpaperVersion", Required = Required.Always)]
        public string WallpaperVersion { get; set; }

        [JsonProperty("WallpaperLocation", Required = Required.Always)]
        public string WallpaperLocation { get; set; }

        [JsonProperty("DescriptionMessage", Required = Required.Always)]
        public string DescriptionMessage { get; set; }

        [JsonProperty("WallpaperAppVersion", Required = Required.Always)]
        public string WallpaperAppVersion { get; set; }
    }
}