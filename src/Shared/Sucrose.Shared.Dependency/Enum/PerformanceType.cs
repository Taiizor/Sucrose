﻿namespace Sucrose.Shared.Dependency.Enum
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
        Upload,
        Download
    }

    internal enum CategoryPerformanceType
    {
        Not,
        Cpu,
        Saver,
        Memory,
        Network,
        Battery
    }
}