using Newtonsoft.Json;
using System.IO;

namespace Sucrose.Shared.Theme.Helper
{
    internal partial class Compatible
    {
        [JsonIgnore]
        public bool State { get; set; }

        [JsonProperty("LoopMode", Required = Required.Default)]
        public string LoopMode { get; set; } = string.Empty;

        [JsonProperty("SystemCpu", Required = Required.Default)]
        public string SystemCpu { get; set; } = string.Empty;

        [JsonProperty("SystemBios", Required = Required.Default)]
        public string SystemBios { get; set; } = string.Empty;

        [JsonProperty("SystemDate", Required = Required.Default)]
        public string SystemDate { get; set; } = string.Empty;

        [JsonProperty("SystemAudio", Required = Required.Default)]
        public string SystemAudio { get; set; } = string.Empty;

        [JsonProperty("VolumeLevel", Required = Required.Default)]
        public string VolumeLevel { get; set; } = string.Empty;

        [JsonProperty("ShuffleMode", Required = Required.Default)]
        public string ShuffleMode { get; set; } = string.Empty;

        [JsonProperty("StretchMode", Required = Required.Default)]
        public string StretchMode { get; set; } = string.Empty;

        [JsonProperty("SystemMemory", Required = Required.Default)]
        public string SystemMemory { get; set; } = string.Empty;

        [JsonProperty("SystemBattery", Required = Required.Default)]
        public string SystemBattery { get; set; } = string.Empty;

        [JsonProperty("SystemGraphic", Required = Required.Default)]
        public string SystemGraphic { get; set; } = string.Empty;

        [JsonProperty("SystemNetwork", Required = Required.Default)]
        public string SystemNetwork { get; set; } = string.Empty;

        [JsonProperty("SystemMotherboard", Required = Required.Default)]
        public string SystemMotherboard { get; set; } = string.Empty;
    }

    internal partial class Compatible
    {
        public static bool CheckJson(string Json)
        {
            try
            {
                JsonConvert.DeserializeObject<Compatible>(Json, Converter.Settings);

                return true;
            }
            catch
            {
                return false;
            }
        }

        public static Compatible FromJson(string Json)
        {
            return JsonConvert.DeserializeObject<Compatible>(Json, Converter.Settings);
        }

        public static Compatible ReadJson(string Path)
        {
            return JsonConvert.DeserializeObject<Compatible>(ReadCompatible(Path), Converter.Settings);
        }

        public static string ReadCompatible(string Path)
        {
            return File.ReadAllText(Path);
        }

        public static void WriteJson(string Path, Compatible Compatible)
        {
            File.WriteAllText(Path, JsonConvert.SerializeObject(Compatible, Converter.Settings));
        }
    }
}