using Newtonsoft.Json;

namespace Sucrose.Shared.Store.Interface
{
    internal class Contents
    {
        /// <summary>
        /// 
        /// </summary>
        [JsonProperty("size", Required = Required.Default)]
        public long Size { get; set; }

        /// <summary>
        /// 
        /// </summary>
        [JsonProperty("sha", Required = Required.Default)]
        public string Sha { get; set; }

        /// <summary>
        /// 
        /// </summary>
        [JsonProperty("url", Required = Required.Default)]
        public string Url { get; set; }

        /// <summary>
        /// 
        /// </summary>
        [JsonProperty("name", Required = Required.Default)]
        public string Name { get; set; }

        /// <summary>
        /// 
        /// </summary>
        [JsonProperty("path", Required = Required.Default)]
        public string Path { get; set; }

        /// <summary>
        /// 
        /// </summary>
        [JsonProperty("type", Required = Required.Default)]
        public string Type { get; set; }

        /// <summary>
        /// 
        /// </summary>
        [JsonProperty("source_url", Required = Required.Default)]
        public string SourceUrl { get; set; }

        /// <summary>
        /// 
        /// </summary>
        [JsonProperty("download_url", Required = Required.Default)]
        public string DownloadUrl { get; set; }

        /// <summary>
        /// 
        /// </summary>
        [JsonProperty("direct_download_url", Required = Required.Default)]
        public string DirectDownloadUrl { get; set; }
    }
}