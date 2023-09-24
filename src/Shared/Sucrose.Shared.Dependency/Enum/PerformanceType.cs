namespace Sucrose.Shared.Dependency.Enum
{
    internal enum PerformanceType
    {
        Close,
        Pause,
        Resume
    }

    internal enum NetworkPerformanceType
    {
        Not,
        Ping,
        Upload,
        Download
    }

    internal enum CategoryPerformanceType
    {
        Not,
        Cpu,
        Saver,
        Memory,
        Remote,
        Network,
        Battery,
        Fullscreen
    }
}