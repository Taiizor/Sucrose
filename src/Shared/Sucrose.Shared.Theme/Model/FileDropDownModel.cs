using Newtonsoft.Json;
using SSTMCM = Sucrose.Shared.Theme.Model.ControlModel;

namespace Sucrose.Shared.Theme.Model
{
    public class FileDropDownModel : SSTMCM
    {
        [JsonProperty("value")]
        public string Value { get; set; }

        [JsonProperty("folder")]
        public string Folder { get; set; }

        [JsonProperty("filter")]
        public string Filter { get; set; }

        public FileDropDownModel() : base("filedropdown") { }
    }
}