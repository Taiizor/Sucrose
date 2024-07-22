namespace Sucrose.Shared.Dependency.Enum
{
    internal enum PerformanceType
    {
        Close,
        Pause,
        Resume
    }

    internal enum PausePerformanceType
    {
        Heavy,
        Light
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
        Gpu,
        Focus,
        Saver,
        Memory,
        Remote,
        Virtual,
        Network,
        Battery,
        Fullscreen
    }
}