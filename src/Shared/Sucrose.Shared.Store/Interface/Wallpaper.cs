using Newtonsoft.Json;

namespace Sucrose.Shared.Store.Interface
{
    internal class Wallpaper
    {
        [JsonProperty("Live", Required = Required.Always)]
        public string Live { get; set; } = string.Empty;

        [JsonProperty("Cover", Required = Required.Always)]
        public string Cover { get; set; } = string.Empty;

        [JsonProperty("Source", Required = Required.Always)]
        public string Source { get; set; } = string.Empty;
    }
}