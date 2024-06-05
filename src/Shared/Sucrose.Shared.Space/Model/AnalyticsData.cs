namespace Sucrose.Shared.Space.Model
{
    internal class AnalyticsData(bool Adult, string User, string Model, string Store, int Startup, string Culture, string Version, string Framework, string Processor, long TotalMemory, string CultureName, int NumberOfCores, string Architecture, string Manufacturer, string DisplayScreen, string CultureDisplay, string OperatingSystem, string ProcessArchitecture, string OperatingSystemBuild, string ProcessorArchitecture, string OperatingSystemArchitecture)
    {
        public bool Adult { get; set; } = Adult;

        public string User { get; set; } = User;

        public string Model { get; set; } = Model;

        public string Store { get; set; } = Store;

        public int Startup { get; set; } = Startup;

        public string Culture { get; set; } = Culture;

        public string Version { get; set; } = Version;

        public string Framework { get; set; } = Framework;

        public string Processor { get; set; } = Processor;

        public long TotalMemory { get; set; } = TotalMemory;

        public string CultureName { get; set; } = CultureName;

        public int NumberOfCores { get; set; } = NumberOfCores;

        public string Architecture { get; set; } = Architecture;

        public string Manufacturer { get; set; } = Manufacturer;

        public string DisplayScreen { get; set; } = DisplayScreen;

        public string CultureDisplay { get; set; } = CultureDisplay;

        public string OperatingSystem { get; set; } = OperatingSystem;

        public string ProcessArchitecture { get; set; } = ProcessArchitecture;

        public string OperatingSystemBuild { get; set; } = OperatingSystemBuild;

        public string ProcessorArchitecture { get; set; } = ProcessorArchitecture;

        public string OperatingSystemArchitecture { get; set; } = OperatingSystemArchitecture;
    }
}