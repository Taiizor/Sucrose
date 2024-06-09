namespace Sucrose.Shared.Space.Model
{
    internal class AnalyticsData(bool Adult, string User, string Model, bool Server, string Store, int Startup, bool Discord, string Update, string Channel, string Culture, string Version, string Framework, string Processor, long TotalMemory, bool Workstation, string CultureName, int NumberOfCores, string Architecture, string Manufacturer, string DisplayScreen, string CultureDisplay, string OperatingSystem, int NumberOfProcessors, string ProcessArchitecture, string OperatingSystemBuild, string ProcessorArchitecture, string OperatingSystemArchitecture)
    {
        public bool Adult { get; set; } = Adult;

        public string User { get; set; } = User;

        public string Model { get; set; } = Model;

        public bool Server { get; set; } = Server;

        public string Store { get; set; } = Store;

        public int Startup { get; set; } = Startup;

        public bool Discord { get; set; } = Discord;

        public string Update { get; set; } = Update;

        public string Channel { get; set; } = Channel;

        public string Culture { get; set; } = Culture;

        public string Version { get; set; } = Version;

        public string Framework { get; set; } = Framework;

        public string Processor { get; set; } = Processor;

        public long TotalMemory { get; set; } = TotalMemory;

        public bool Workstation { get; set; } = Workstation;

        public string CultureName { get; set; } = CultureName;

        public int NumberOfCores { get; set; } = NumberOfCores;

        public string Architecture { get; set; } = Architecture;

        public string Manufacturer { get; set; } = Manufacturer;

        public string DisplayScreen { get; set; } = DisplayScreen;

        public string CultureDisplay { get; set; } = CultureDisplay;

        public string OperatingSystem { get; set; } = OperatingSystem;

        public int NumberOfProcessors { get; set; } = NumberOfProcessors;

        public string ProcessArchitecture { get; set; } = ProcessArchitecture;

        public string OperatingSystemBuild { get; set; } = OperatingSystemBuild;

        public string ProcessorArchitecture { get; set; } = ProcessorArchitecture;

        public string OperatingSystemArchitecture { get; set; } = OperatingSystemArchitecture;
    }
}