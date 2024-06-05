using SSDEST = Sucrose.Shared.Dependency.Enum.StoreType;
using SEDST = Skylark.Enum.DisplayScreenType;
using SEOST = Skylark.Enum.OperatingSystemType;

namespace Sucrose.Shared.Space.Model
{
    internal class AnalyticsData(bool Adult, string Model, string Culture, string Startup, string Version, string Framework, SSDEST StoreType, string Architecture, SEDST DisplayScreenType, SEOST OperatingSystemType, string ProcessArchitecture, string OperatingSystemBuild, string ProcessorArchitecture, string OperatingSystemArchitecture)
    {
        public bool Adult { get; set; } = Adult;

        public string Model { get; set; } = Model;

        public string Culture { get; set; } = Culture;

        public string Startup { get; set; } = Startup;

        public string Version { get; set; } = Version;

        public string Framework { get; set; } = Framework;

        public SSDEST StoreType { get; set; } = StoreType;

        public string Architecture { get; set; } = Architecture;

        public SEDST DisplayScreenType { get; set; } = DisplayScreenType;

        public SEOST OperatingSystemType { get; set; } = OperatingSystemType;

        public string ProcessArchitecture { get; set; } = ProcessArchitecture;

        public string OperatingSystemBuild { get; set; } = OperatingSystemBuild;

        public string ProcessorArchitecture { get; set; } = ProcessArchitecture;

        public string OperatingSystemArchitecture { get; set; } = OperatingSystemArchitecture;
    }
}