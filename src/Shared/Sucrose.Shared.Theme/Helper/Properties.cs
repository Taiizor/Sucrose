using Newtonsoft.Json;
using System.IO;
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
    }

    internal partial class Properties
    {
        public static bool CheckJson(string Json)
        {
            try
            {
                JsonConvert.DeserializeObject<Properties>(Json, Converter.Settings);

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
            return JsonConvert.DeserializeObject<Properties>(ReadProperties(Path), Converter.Settings);
        }

        public static string ReadProperties(string Path)
        {
            return File.ReadAllText(Path);
        }

        public static void WriteJson(string Path, Properties Properties)
        {
            File.WriteAllText(Path, JsonConvert.SerializeObject(Properties, Converter.Settings));
        }
    }
}