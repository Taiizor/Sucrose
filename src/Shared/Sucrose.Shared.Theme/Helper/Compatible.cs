using Newtonsoft.Json;
using System.IO;

namespace Sucrose.Shared.Theme.Helper
{
    internal partial class Compatible
    {
        public bool State { get; set; }

        [JsonProperty("TriggerTime", Required = Required.Always)]
        public int TriggerTime { get; set; }

        [JsonProperty("LoopMode", Required = Required.Default)]
        public string LoopMode { get; set; }

        [JsonProperty("VolumeLevel", Required = Required.Default)]
        public string VolumeLevel { get; set; }

        [JsonProperty("ShuffleMode", Required = Required.Default)]
        public string ShuffleMode { get; set; }

        [JsonProperty("StretchMode", Required = Required.Default)]
        public string StretchMode { get; set; }

        [JsonProperty("ComputerDate", Required = Required.Default)]
        public string ComputerDate { get; set; }
    }

    internal partial class Compatible
    {
        public static Compatible FromJson(string Json)
        {
            return JsonConvert.DeserializeObject<Compatible>(Json, Converter.Settings);
        }

        public static Compatible ReadJson(string Json)
        {
            return JsonConvert.DeserializeObject<Compatible>(File.ReadAllText(Json), Converter.Settings);
        }
    }
}