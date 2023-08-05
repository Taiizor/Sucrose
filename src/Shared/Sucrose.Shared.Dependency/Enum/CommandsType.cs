namespace Sucrose.Shared.Dependency.Enum
{
    internal enum CommandsType
    {
        Log,
        Kill,
        Live,
        Test,
        Temp,
        Update,
        Import,
        Export,
        Report,
        Startup,
        StartupM,
        StartupP,
        Scheduler,
        Interface
    }

    internal enum SchedulerCommandsType
    {
        Create,
        Enable,
        Delete,
        Disable
    }
}