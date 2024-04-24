using Newtonsoft.Json;
using SSTMCM = Sucrose.Shared.Theme.Model.ControlModel;

namespace Sucrose.Shared.Theme.Model
{
    public class FileDropDownModel : SSTMCM
    {
        [JsonProperty("desc", Required = Required.Always)]
        public string Desc { get; set; }

        [JsonProperty("value", Required = Required.Always)]
        public string Value { get; set; }

        [JsonProperty("title", Required = Required.Always)]
        public string Title { get; set; }

        [JsonProperty("folder", Required = Required.Always)]
        public string Folder { get; set; }

        [JsonProperty("filter", Required = Required.Always)]
        public string Filter { get; set; }

        public FileDropDownModel() : base("filedropdown") { }
    }
}