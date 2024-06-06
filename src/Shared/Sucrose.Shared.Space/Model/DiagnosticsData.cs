using Newtonsoft.Json.Linq;

namespace Sucrose.Shared.Space.Model
{
    internal class DiagnosticsData(Guid Id, Guid Sid, string App, Guid AppId, string User, string Model, bool Server, string Culture, string Version, string Framework, JObject Exception, bool Workstation, string CultureName, string Architecture, string Manufacturer, string CultureDisplay, string OperatingSystem, string ProcessArchitecture, string OperatingSystemBuild, string ProcessorArchitecture, string OperatingSystemArchitecture)
    {
        public Guid Id { get; set; } = Id;

        public Guid Sid { get; set; } = Sid;

        public string App { get; set; } = App;

        public Guid AppId { get; set; } = AppId;

        public string User { get; set; } = User;

        public string Model { get; set; } = Model;

        public bool Server { get; set; } = Server;

        public string Culture { get; set; } = Culture;

        public string Version { get; set; } = Version;

        public string Framework { get; set; } = Framework;

        public JObject Exception { get; set; } = Exception;

        public bool Workstation { get; set; } = Workstation;

        public string CultureName { get; set; } = CultureName;

        public string Architecture { get; set; } = Architecture;

        public string Manufacturer { get; set; } = Manufacturer;

        public string CultureDisplay { get; set; } = CultureDisplay;

        public string OperatingSystem { get; set; } = OperatingSystem;

        public string ProcessArchitecture { get; set; } = ProcessArchitecture;

        public string OperatingSystemBuild { get; set; } = OperatingSystemBuild;

        public string ProcessorArchitecture { get; set; } = ProcessorArchitecture;

        public string OperatingSystemArchitecture { get; set; } = OperatingSystemArchitecture;
    }
}