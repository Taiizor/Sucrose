using Newtonsoft.Json;
using SSTMCM = Sucrose.Shared.Theme.Model.ControlModel;

namespace Sucrose.Shared.Theme.Model
{
    public class CheckBoxModel : SSTMCM
    {
        [JsonProperty("value")]
        public bool Value { get; set; }

        public CheckBoxModel() : base("checkbox") { }
    }
}