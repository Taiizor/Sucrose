using Newtonsoft.Json;

namespace Sucrose.Shared.Theme.Model
{
    public class ControlModel
    {
        [JsonIgnore]
        [JsonProperty("name", Required = Required.Default)]
        public string Name { get; set; }

        [JsonProperty("text", Required = Required.Always)]
        public string Text { get; set; }

        [JsonProperty("type", Required = Required.Always)]
        public string Type { get; }

        [JsonProperty("help", Required = Required.Default)]
        public string Help { get; set; }

        protected ControlModel(string Type)
        {
            this.Type = Type;
        }
    }
}