using SSDEST = Sucrose.Shared.Dependency.Enum.StoreType;
using SEDST = Skylark.Enum.DisplayScreenType;
using SEOST = Skylark.Enum.OperatingSystemType;

namespace Sucrose.Shared.Space.Model
{
    internal class DiagnosticsData(Guid Id, string Sid, Guid AppId, string Model, string Culture, string Version, Exception Exception, SEOST OperatingSystemType, string ProcessArchitecture, string OperatingSystemBuild, string ProcessorArchitecture, string OperatingSystemArchitecture)
    {
        public Guid Id { get; set; } = Id;

        public string Sid { get; set; } = Sid;

        public Guid AppId { get; set; } = AppId;

        public string Model { get; set; } = Model;

        public string Culture { get; set; } = Culture;

        public string Version { get; set; } = Version;

        public Exception Exception { get; set; } = Exception;

        public SEOST OperatingSystemType { get; set; } = OperatingSystemType;

        public string ProcessArchitecture { get; set; } = ProcessArchitecture;

        public string OperatingSystemBuild { get; set; } = OperatingSystemBuild;

        public string ProcessorArchitecture { get; set; } = ProcessorArchitecture;

        public string OperatingSystemArchitecture { get; set; } = OperatingSystemArchitecture;
    }
}