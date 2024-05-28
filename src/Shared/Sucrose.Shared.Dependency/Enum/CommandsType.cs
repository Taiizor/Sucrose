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
        Cycyling,
        Property,
        StartupM,
        StartupP,
        Official,
        Watchdog,
        PropertyA,
        Scheduler,
        Interface,
        Repository,
        Discussions,
        Backgroundog
    }

    internal enum ArgumentCommandsType
    {
        SystemSetting,
        GeneralSetting
    }

    internal enum SchedulerCommandsType
    {
        Create,
        Enable,
        Delete,
        Disable
    }
}