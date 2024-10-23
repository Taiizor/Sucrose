using Newtonsoft.Json;
using Newtonsoft.Json.Linq;
using SSSHF = Sucrose.Shared.Space.Helper.Filing;

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

        [JsonProperty("ThemeType", Required = Required.Default)]
        public string ThemeType { get; set; } = string.Empty;

        [JsonProperty("SystemBios", Required = Required.Default)]
        public string SystemBios { get; set; } = string.Empty;

        [JsonProperty("SystemDate", Required = Required.Default)]
        public string SystemDate { get; set; } = string.Empty;

        [JsonProperty("ShuffleMode", Required = Required.Default)]
        public string ShuffleMode { get; set; } = string.Empty;

        [JsonProperty("StretchMode", Required = Required.Default)]
        public string StretchMode { get; set; } = string.Empty;

        [JsonProperty("SystemAudio", Required = Required.Default)]
        public string SystemAudio { get; set; } = string.Empty;

        [JsonProperty("SystemTheme", Required = Required.Default)]
        public string SystemTheme { get; set; } = string.Empty;

        [JsonProperty("VolumeLevel", Required = Required.Default)]
        public string VolumeLevel { get; set; } = string.Empty;

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
        public static string Read(string Path)
        {
            string Content = SSSHF.Read(Path);

            return string.IsNullOrWhiteSpace(Content) ? string.Empty : Content;
        }

        public static bool FromCheck(string Json)
        {
            try
            {
                JsonConvert.DeserializeObject<Compatible>(Json, Converter.Settings);

                JToken.Parse(Json);

                return true;
            }
            catch
            {
                return false;
            }
        }

        public static bool ReadCheck(string Path)
        {
            try
            {
                string Content = Read(Path);

                JsonConvert.DeserializeObject<Compatible>(Content, Converter.Settings);

                JToken.Parse(Content);

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
            return JsonConvert.DeserializeObject<Compatible>(Read(Path), Converter.Settings);
        }

        public static void Write(string Path, Compatible Json)
        {
            SSSHF.Write(Path, JsonConvert.SerializeObject(Json, Converter.Settings));
        }
    }
}