using Newtonsoft.Json;
using Newtonsoft.Json.Linq;
using SSTMBM = Sucrose.Shared.Theme.Model.ButtonModel;
using SSTMCBM = Sucrose.Shared.Theme.Model.CheckBoxModel;
using SSTMCM = Sucrose.Shared.Theme.Model.ControlModel;
using SSTMCPM = Sucrose.Shared.Theme.Model.ColorPickerModel;
using SSTMDDM = Sucrose.Shared.Theme.Model.DropDownModel;
using SSTMFDDM = Sucrose.Shared.Theme.Model.FileDropDownModel;
using SSTMLM = Sucrose.Shared.Theme.Model.LabelModel;
using SSTMNBM = Sucrose.Shared.Theme.Model.NumberBoxModel;
using SSTMPBM = Sucrose.Shared.Theme.Model.PasswordBoxModel;
using SSTMSM = Sucrose.Shared.Theme.Model.SliderModel;
using SSTMTBM = Sucrose.Shared.Theme.Model.TextBoxModel;

namespace Sucrose.Shared.Theme.Converter
{
    public class ModelConverter : JsonConverter
    {
        public override bool CanConvert(Type objectType)
        {
            return objectType == typeof(SSTMCM);
        }

        public override object ReadJson(JsonReader reader, Type objectType, object existingValue, JsonSerializer serializer)
        {
            JObject jsonObject = JObject.Load(reader);
            string type = jsonObject["type"]?.Value<string>();
            string text = jsonObject["text"]?.Value<string>();

            if (type?.ToLower() == "label" && string.IsNullOrEmpty(text))
            {
                jsonObject["text"] = string.Empty;
            }

            SSTMCM control = type?.ToLower() switch
            {
                "label" => jsonObject.ToObject<SSTMLM>(),
                "button" => jsonObject.ToObject<SSTMBM>(),
                "slider" => jsonObject.ToObject<SSTMSM>(),
                "textbox" => jsonObject.ToObject<SSTMTBM>(),
                "checkbox" => jsonObject.ToObject<SSTMCBM>(),
                "dropdown" => jsonObject.ToObject<SSTMDDM>(),
                "numberbox" => jsonObject.ToObject<SSTMNBM>(),
                "colorpicker" => jsonObject.ToObject<SSTMCPM>(),
                "passwordbox" => jsonObject.ToObject<SSTMPBM>(),
                "filedropdown" => jsonObject.ToObject<SSTMFDDM>(),
                _ => throw new NotSupportedException($"Control type '{type}' is not supported."),
            };

            if (string.IsNullOrEmpty(control.Name))
            {
                control.Name = reader.Path.Split('.').Last();
            }

            return control;
        }

        public override void WriteJson(JsonWriter writer, object value, JsonSerializer serializer)
        {
            writer.WriteValue(value.ToString());
        }
    }
}