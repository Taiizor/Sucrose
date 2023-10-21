using Newtonsoft.Json;
using System.IO;

namespace Sucrose.Shared.Theme.Helper
{
    internal partial class Properties
    {
        public bool State { get; set; }

        [JsonProperty("PropertyListener", Required = Required.Always)]
        public string PropertyListener { get; set; }

        [JsonProperty("PropertyList", Required = Required.Always)]
        public Dictionary<string, object> PropertyList { get; set; }
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

        public static Properties ReadJson(string Json)
        {
            return JsonConvert.DeserializeObject<Properties>(File.ReadAllText(Json), Converter.Settings);
        }
    }
}