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

        [JsonProperty("SystemCpu", Required = Required.Default)]
        public string SystemCpu { get; set; }

        [JsonProperty("SystemBios", Required = Required.Default)]
        public string SystemBios { get; set; }

        [JsonProperty("SystemDate", Required = Required.Default)]
        public string SystemDate { get; set; }

        [JsonProperty("VolumeLevel", Required = Required.Default)]
        public string VolumeLevel { get; set; }

        [JsonProperty("ShuffleMode", Required = Required.Default)]
        public string ShuffleMode { get; set; }

        [JsonProperty("StretchMode", Required = Required.Default)]
        public string StretchMode { get; set; }

        [JsonProperty("SystemMemory", Required = Required.Default)]
        public string SystemMemory { get; set; }

        [JsonProperty("SystemBattery", Required = Required.Default)]
        public string SystemBattery { get; set; }

        [JsonProperty("SystemNetwork", Required = Required.Default)]
        public string SystemNetwork { get; set; }

        [JsonProperty("SystemMotherboard", Required = Required.Default)]
        public string SystemMotherboard { get; set; }
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