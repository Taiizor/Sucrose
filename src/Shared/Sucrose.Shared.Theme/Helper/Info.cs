using Newtonsoft.Json;
using Newtonsoft.Json.Linq;
using SSDEWT = Sucrose.Shared.Dependency.Enum.WallpaperType;
using SSSHF = Sucrose.Shared.Space.Helper.Filing;

namespace Sucrose.Shared.Theme.Helper
{
    internal partial class Info
    {
        [JsonProperty("AppVersion", Required = Required.Always)]
        public Version AppVersion { get; set; } = new(0, 0, 0, 0);

        [JsonProperty("Version", Required = Required.Always)]
        public Version Version { get; set; } = new(1, 0, 0, 0);

        [JsonProperty("Title", Required = Required.Always)]
        public string Title { get; set; } = string.Empty;

        [JsonProperty("Thumbnail", Required = Required.Always)]
        public string Thumbnail { get; set; } = string.Empty;

        [JsonProperty("Preview", Required = Required.Always)]
        public string Preview { get; set; } = string.Empty;

        [JsonProperty("Description", Required = Required.Always)]
        public string Description { get; set; } = string.Empty;

        [JsonProperty("Author", Required = Required.Always)]
        public string Author { get; set; } = string.Empty;

        [JsonProperty("License", Required = Required.AllowNull)]
        public string License { get; set; } = string.Empty;

        [JsonProperty("Contact", Required = Required.Always)]
        public string Contact { get; set; } = string.Empty;

        [JsonProperty("Type", Required = Required.Always)]
        public SSDEWT Type { get; set; } = SSDEWT.Gif;

        [JsonProperty("Source", Required = Required.Always)]
        public string Source { get; set; } = string.Empty;

        [JsonProperty("Tags", Required = Required.AllowNull)]
        public string[] Tags { get; set; } = Array.Empty<string>();

        [JsonProperty("Arguments", Required = Required.AllowNull)]
        public string Arguments { get; set; } = string.Empty;
    }

    internal partial class Info
    {
        public static string Read(string Path)
        {
            string Content = SSSHF.Read(Path);

            return string.IsNullOrWhiteSpace(Content) ? string.Empty : Content;
        }

        public static Info FromJson(string Json)
        {
            return JsonConvert.DeserializeObject<Info>(Json, Converter.Settings);
        }

        public static Info ReadJson(string Path)
        {
            return JsonConvert.DeserializeObject<Info>(Read(Path), Converter.Settings);
        }

        public static bool FromCheck(string Json)
        {
            try
            {
                JsonConvert.DeserializeObject<Info>(Json, Converter.Settings);

                JToken.Parse(Json);

                return true;
            }
            catch
            {
                return false;
            }
        }

        public static bool ReadCheck(string Path)
        {
            try
            {
                string Content = Read(Path);

                JsonConvert.DeserializeObject<Info>(Content, Converter.Settings);

                JToken.Parse(Content);

                return true;
            }
            catch
            {
                return false;
            }
        }

        public static void Write(string Path, Info Json)
        {
            SSSHF.Write(Path, JsonConvert.SerializeObject(Json, Converter.Settings));
        }
    }
}