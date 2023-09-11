namespace Sucrose.Shared.Dependency.Enum
{
    internal enum CommandsType
    {
        Log,
        Kill,
        Live,
        Test,
        Temp,
        Bundle,
        Update,
        Import,
        Export,
        Report,
        Publish,
        Startup,
        StartupM,
        StartupP,
        Scheduler,
        Interface
    }

    internal enum ArgumentCommandsType
    {
        Setting
    }

    internal enum SchedulerCommandsType
    {
        Create,
        Enable,
        Delete,
        Disable
    }
}