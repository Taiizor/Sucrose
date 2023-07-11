using Newtonsoft.Json;

namespace Sucrose.Manager.Converter
{
    public class EnumConverter : JsonConverter
    {
        public override bool CanConvert(Type objectType)
        {
            return objectType.IsEnum;
        }

        public override object ReadJson(JsonReader reader, Type objectType, object existingValue, JsonSerializer serializer)
        {
            if (reader.TokenType == JsonToken.String)
            {
                string enumValue = (string)reader.Value;
                return Enum.Parse(objectType, enumValue, true);
            }

            throw new JsonSerializationException("Invalid enum value.");
        }

        public override void WriteJson(JsonWriter writer, object value, JsonSerializer serializer)
        {
            writer.WriteValue(value.ToString());
        }
    }
}