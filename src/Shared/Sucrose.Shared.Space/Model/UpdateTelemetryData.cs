using Newtonsoft.Json;

namespace Sucrose.Shared.Space.Model
{
    internal class UpdateTelemetryData
    {
        [JsonProperty("SilentMode", Required = Required.Always)]
        public bool SilentMode { get; set; }

        [JsonProperty("AppVersion", Required = Required.Always)]
        public string AppVersion { get; set; }
    }
}