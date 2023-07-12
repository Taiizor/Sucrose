﻿using Newtonsoft.Json;
using Newtonsoft.Json.Converters;
using System.Globalization;
using System.IO;
using SDEWT = Sucrose.Dependency.Enum.WallpaperType;

namespace Sucrose.Theme.Shared.Helper
{
    internal partial class Info
    {
        [JsonProperty("AppVersion", Required = Required.Always)]
        public Version AppVersion { get; set; }

        [JsonProperty("Version", Required = Required.Always)]
        public Version Version { get; set; }

        [JsonProperty("Title", Required = Required.Always)]
        public string Title { get; set; }

        [JsonProperty("Thumbnail", Required = Required.Always)]
        public string Thumbnail { get; set; }

        [JsonProperty("Preview", Required = Required.Always)]
        public string Preview { get; set; }

        [JsonProperty("Description", Required = Required.Always)]
        public string Description { get; set; }

        [JsonProperty("Author", Required = Required.AllowNull)]
        public string Author { get; set; }

        [JsonProperty("License", Required = Required.AllowNull)]
        public string License { get; set; }

        [JsonProperty("Contact", Required = Required.AllowNull)]
        public string Contact { get; set; }

        [JsonProperty("Type", Required = Required.Always)]
        public SDEWT Type { get; set; }

        [JsonProperty("Source", Required = Required.Always)]
        public string Source { get; set; }

        [JsonProperty("Arguments", Required = Required.AllowNull)]
        public string Arguments { get; set; }
    }

    internal partial class Info
    {
        public static Info FromJson(string Json)
        {
            return JsonConvert.DeserializeObject<Info>(Json, Converter.Settings);
        }

        public static Info ReadJson(string Json)
        {
            return JsonConvert.DeserializeObject<Info>(File.ReadAllText(Json), Converter.Settings);
        }
    }

    internal static class Serialize
    {
        public static string ToJson(this Info self)
        {
            return JsonConvert.SerializeObject(self, Converter.Settings);
        }
    }

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