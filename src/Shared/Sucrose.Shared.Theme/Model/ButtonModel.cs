using Newtonsoft.Json;
using SSTMCM = Sucrose.Shared.Theme.Model.ControlModel;

namespace Sucrose.Shared.Theme.Model
{
    public class ButtonModel : SSTMCM
    {
        [JsonProperty("value", Required = Required.AllowNull)]
        public string Value { get; set; }

        [JsonProperty("command", Required = Required.Always)]
        public string Command { get; set; }

        public ButtonModel() : base("button") { }
    }
}