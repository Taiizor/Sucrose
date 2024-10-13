using Newtonsoft.Json;
using Newtonsoft.Json.Converters;
using System.Globalization;
using SSTCMC = Sucrose.Shared.Theme.Converter.ModelConverter;
using SSTCVC = Sucrose.Shared.Theme.Converter.VersionConverter;

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
                new SSTCVC(),
                new IsoDateTimeConverter
                {
                    DateTimeStyles = DateTimeStyles.AssumeUniversal
                }
            }
        };
    }
}