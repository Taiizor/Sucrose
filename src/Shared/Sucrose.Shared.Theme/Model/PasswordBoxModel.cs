using Newtonsoft.Json;
using SSTMCM = Sucrose.Shared.Theme.Model.ControlModel;

namespace Sucrose.Shared.Theme.Model
{
    public class PasswordBoxModel : SSTMCM
    {
        [JsonProperty("value")]
        public string Value { get; set; }

        public PasswordBoxModel() : base("passwordbox") { }
    }
}