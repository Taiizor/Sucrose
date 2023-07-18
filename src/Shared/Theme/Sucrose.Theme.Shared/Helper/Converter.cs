using Newtonsoft.Json;
using Newtonsoft.Json.Converters;
using System.Globalization;

namespace Sucrose.Theme.Shared.Helper
{
    internal static class Converter
    {
        public static readonly JsonSerializerSettings Settings = new()
        {
            MetadataPropertyHandling = MetadataPropertyHandling.Ignore,
            DateParseHandling = DateParseHandling.None,
            Converters =
            {
                new IsoDateTimeConverter
                {
                    DateTimeStyles = DateTimeStyles.AssumeUniversal
                }
            },
        };
    }
}