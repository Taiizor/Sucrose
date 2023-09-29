namespace Sucrose.Shared.Dependency.Enum
{
    internal enum CommandsType
    {
        Log,
        Kill,
        Live,
        Test,
        Temp,
        Wiki,
        Reset,
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
        Interface,
        Backgroundog
    }

    internal enum ArgumentCommandsType
    {
        Setting,
        SystemSetting
    }

    internal enum SchedulerCommandsType
    {
        Create,
        Enable,
        Delete,
        Disable
    }
}