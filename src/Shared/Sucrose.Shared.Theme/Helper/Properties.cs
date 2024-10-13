using Newtonsoft.Json;
using Newtonsoft.Json.Linq;
using SSSHF = Sucrose.Shared.Space.Helper.Filing;
using SSTMCM = Sucrose.Shared.Theme.Model.ControlModel;

namespace Sucrose.Shared.Theme.Helper
{
    internal partial class Properties
    {
        [JsonIgnore]
        public bool State { get; set; }

        [JsonProperty("PropertyListener", Required = Required.Always)]
        public string PropertyListener { get; set; } = string.Empty;

        [JsonProperty("PropertyList", Required = Required.Always)]
        public Dictionary<string, SSTMCM> PropertyList { get; set; } = new();

        [JsonProperty("PropertyLocalization", Required = Required.Default)]
        public Dictionary<string, Dictionary<string, string>> PropertyLocalization { get; set; } = null;
    }

    internal partial class Properties
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
                JsonConvert.DeserializeObject<Properties>(Json, Converter.Settings);

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

                JsonConvert.DeserializeObject<Properties>(Content, Converter.Settings);

                JToken.Parse(Content);

                return true;
            }
            catch
            {
                return false;
            }
        }

        public static Properties FromJson(string Json)
        {
            return JsonConvert.DeserializeObject<Properties>(Json, Converter.Settings);
        }

        public static Properties ReadJson(string Path)
        {
            return JsonConvert.DeserializeObject<Properties>(Read(Path), Converter.Settings);
        }

        public static void Write(string Path, Properties Json)
        {
            SSSHF.Write(Path, JsonConvert.SerializeObject(Json, Converter.Settings));
        }
    }
}