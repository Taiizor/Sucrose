using Newtonsoft.Json;

namespace Sucrose.Shared.Store.Interface
{
    internal class Root
    {
        [JsonProperty("Categories", Required = Required.Always)]
        public Dictionary<string, Category> Categories { get; set; } = new();
    }
}