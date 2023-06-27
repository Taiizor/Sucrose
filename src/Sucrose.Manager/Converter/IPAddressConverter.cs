using Newtonsoft.Json;
using System;
using System.Net;

namespace Sucrose.Manager.Converter
{
    public class IPAddressConverter : JsonConverter
    {
        public override bool CanConvert(Type objectType)
        {
            return objectType == typeof(IPAddress);
        }

        public override object ReadJson(JsonReader reader, Type objectType, object existingValue, JsonSerializer serializer)
        {
            string value = (string)reader.Value;
            return IPAddress.Parse(value);
        }

        public override void WriteJson(JsonWriter writer, object value, JsonSerializer serializer)
        {
            IPAddress ipAddress = (IPAddress)value;
            writer.WriteValue(ipAddress.ToString());
        }
    }
}