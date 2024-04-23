using Newtonsoft.Json;
using Newtonsoft.Json.Converters;
using System.Globalization;
using SSTCMC = Sucrose.Shared.Theme.Converter.ModelConverter;

namespace Sucrose.Shared.Theme.Helper
{
    internal static class Converter
    {
        public static readonly JsonSerializerSettings Settings = new()
        {
            MetadataPropertyHandling = MetadataPropertyHandling.Ignore,
            DateParseHandling = DateParseHandling.None,
            Formatting = Formatting.Indented,
            Converters =
            {
                new SSTCMC(),
                new VersionConverter(),
                new IsoDateTimeConverter
                {
                    DateTimeStyles = DateTimeStyles.AssumeUniversal
                }
            }
        };
    }
}