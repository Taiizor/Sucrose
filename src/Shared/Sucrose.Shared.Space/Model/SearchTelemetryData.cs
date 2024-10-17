using Newtonsoft.Json;

namespace Sucrose.Shared.Space.Model
{
    internal class SearchTelemetryData
    {
        [JsonProperty("QueryText", Required = Required.Always)]
        public string QueryText { get; set; }

        [JsonProperty("ActivePage", Required = Required.Always)]
        public string ActivePage { get; set; }

        [JsonProperty("AppVersion", Required = Required.Always)]
        public string AppVersion { get; set; }
    }
}