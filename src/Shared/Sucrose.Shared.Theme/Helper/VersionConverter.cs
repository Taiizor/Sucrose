using Newtonsoft.Json;
using Newtonsoft.Json.Linq;

namespace Sucrose.Shared.Theme.Helper
{
    internal class VersionConverter : JsonConverter<Version>
    {
        public override Version ReadJson(JsonReader reader, Type objectType, Version existingValue, bool hasExistingValue, JsonSerializer serializer)
        {
            if (reader.TokenType == JsonToken.String)
            {
                string versionString = (string)reader.Value;

                return new Version(versionString);
            }
            else if (reader.TokenType == JsonToken.StartObject)
            {
                JObject jObject = JObject.Load(reader);

                int major = (int)jObject["Major"];
                int minor = (int)jObject["Minor"];
                int build = (int)jObject["Build"];
                int revision = (int)jObject["Revision"];

                return new Version(major, minor, build, revision);
            }

            return new(1, 0, 0, 0);
        }

        public override void WriteJson(JsonWriter writer, Version value, JsonSerializer serializer)
        {
            serializer.Serialize(writer, value.ToString());
        }
    }
}