using Newtonsoft.Json;
using Newtonsoft.Json.Linq;

namespace Sucrose.Shared.Space.Model
{
    internal class ThrowExceptionData
    {
        [JsonProperty("Id", Required = Required.Always)]
        public Guid Id { get; set; }

        [JsonProperty("Sid", Required = Required.Always)]
        public Guid Sid { get; set; }

        [JsonProperty("AppId", Required = Required.Always)]
        public Guid AppId { get; set; }

        [JsonProperty("IsServer", Required = Required.Always)]
        public bool IsServer { get; set; }

        [JsonProperty("AppName", Required = Required.Always)]
        public string AppName { get; set; }

        [JsonProperty("UserName", Required = Required.Always)]
        public string UserName { get; set; }

        [JsonProperty("AppVersion", Required = Required.Always)]
        public string AppVersion { get; set; }

        [JsonProperty("Exception", Required = Required.Always)]
        public JObject Exception { get; set; }

        [JsonProperty("CultureCode", Required = Required.Always)]
        public string CultureCode { get; set; }

        [JsonProperty("CultureName", Required = Required.Always)]
        public string CultureName { get; set; }

        [JsonProperty("DeviceModel", Required = Required.Always)]
        public string DeviceModel { get; set; }

        [JsonProperty("IsWorkstation", Required = Required.Always)]
        public bool IsWorkstation { get; set; }

        [JsonProperty("AppFramework", Required = Required.Always)]
        public string AppFramework { get; set; }

        [JsonProperty("CultureDisplay", Required = Required.Always)]
        public string CultureDisplay { get; set; }

        [JsonProperty("AppArchitecture", Required = Required.Always)]
        public string AppArchitecture { get; set; }

        [JsonProperty("OperatingSystem", Required = Required.Always)]
        public string OperatingSystem { get; set; }

        [JsonProperty("ManufacturerBrand", Required = Required.Always)]
        public string ManufacturerBrand { get; set; }

        [JsonProperty("ProcessArchitecture", Required = Required.Always)]
        public string ProcessArchitecture { get; set; }

        [JsonProperty("OperatingSystemBuild", Required = Required.Always)]
        public string OperatingSystemBuild { get; set; }

        [JsonProperty("ProcessorArchitecture", Required = Required.Always)]
        public string ProcessorArchitecture { get; set; }

        [JsonProperty("OperatingSystemArchitecture", Required = Required.Always)]
        public string OperatingSystemArchitecture { get; set; }
    }
}