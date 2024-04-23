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
        Wizard,
        Publish,
        Startup,
        Property,
        StartupM,
        StartupP,
        Official,
        Watchdog,
        Scheduler,
        Interface,
        Repository,
        Discussions,
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