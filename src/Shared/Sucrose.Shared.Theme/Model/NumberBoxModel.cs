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

        [JsonProperty("value")]
        public double Value { get; set; }

        // Default value 1, otherwise if missing it will be 0 and crash on moving slider.
        [JsonProperty("step")]
        public double Step { get; set; } = 1d;

        public NumberBoxModel() : base("slider") { }
    }
}