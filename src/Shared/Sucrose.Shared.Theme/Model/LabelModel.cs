using Newtonsoft.Json;
using SSTMCM = Sucrose.Shared.Theme.Model.ControlModel;

namespace Sucrose.Shared.Theme.Model
{
    public class LabelModel : SSTMCM
    {
        [JsonProperty("value")]
        public string Value { get; set; }

        public LabelModel() : base("label") { }
    }
}