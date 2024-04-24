using Newtonsoft.Json;
using SSTMCM = Sucrose.Shared.Theme.Model.ControlModel;

namespace Sucrose.Shared.Theme.Model
{
    public class NumberBoxModel : SSTMCM
    {
        [JsonProperty("min")]
        public double Min { get; set; }

        [JsonProperty("max")]
        public double Max { get; set; }

        [JsonProperty("places")]
        public int Places { get; set; }

        [JsonProperty("value")]
        public double Value { get; set; }

        public NumberBoxModel() : base("numberbox") { }
    }
}