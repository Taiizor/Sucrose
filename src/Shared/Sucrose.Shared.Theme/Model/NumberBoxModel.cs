using Newtonsoft.Json;
using SSTMCM = Sucrose.Shared.Theme.Model.ControlModel;

namespace Sucrose.Shared.Theme.Model
{
    public class NumberBoxModel : SSTMCM
    {
        [JsonProperty("min", Required = Required.Always)]
        public double Min { get; set; }

        [JsonProperty("max", Required = Required.Always)]
        public double Max { get; set; }

        [JsonProperty("places", Required = Required.Always)]
        public int Places { get; set; }

        [JsonProperty("value", Required = Required.Always)]
        public double Value { get; set; }

        public NumberBoxModel() : base("numberbox") { }
    }
}