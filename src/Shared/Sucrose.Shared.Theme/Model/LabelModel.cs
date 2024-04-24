using Newtonsoft.Json;
using SSTMCM = Sucrose.Shared.Theme.Model.ControlModel;

namespace Sucrose.Shared.Theme.Model
{
    public class LabelModel : SSTMCM
    {
        [JsonProperty("value", Required = Required.Always)]
        public string Value { get; set; }

        public LabelModel() : base("label") { }
    }
}