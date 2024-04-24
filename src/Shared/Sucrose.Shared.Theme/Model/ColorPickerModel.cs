using Newtonsoft.Json;
using SSTMCM = Sucrose.Shared.Theme.Model.ControlModel;

namespace Sucrose.Shared.Theme.Model
{
    public class ColorPickerModel : SSTMCM
    {
        [JsonProperty("value", Required = Required.Always)]
        public string Value { get; set; }

        public ColorPickerModel() : base("colorpicker") { }
    }
}