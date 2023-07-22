using Newtonsoft.Json;
using System.IO;

namespace Sucrose.Shared.Theme.Helper
{
    internal partial class Properties
    {
        [JsonProperty("Test", Required = Required.Always)]
        public int Test { get; set; }
    }

    internal partial class Properties
    {
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