using Newtonsoft.Json;
using SSTMCM = Sucrose.Shared.Theme.Model.ControlModel;

namespace Sucrose.Shared.Theme.Model
{
    public class SliderModel : SSTMCM
    {
        [JsonProperty("min", Required = Required.Always)]
        public double Min { get; set; }

        [JsonProperty("max", Required = Required.Always)]
        public double Max { get; set; }

        [JsonProperty("step", Required = Required.Always)]
        public double Step { get; set; }

        [JsonProperty("value", Required = Required.Always)]
        public double Value { get; set; }

        public SliderModel() : base("slider") { }
    }
}