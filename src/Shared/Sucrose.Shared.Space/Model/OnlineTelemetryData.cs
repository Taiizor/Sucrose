using Newtonsoft.Json;

namespace Sucrose.Shared.Space.Model
{
    internal class OnlineTelemetryData
    {
        [JsonProperty("Time", Required = Required.Always)]
        public int Time { get; set; }

        [JsonProperty("AppVersion", Required = Required.Always)]
        public string AppVersion { get; set; }
    }
}