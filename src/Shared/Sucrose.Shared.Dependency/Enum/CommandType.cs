namespace Sucrose.Shared.Dependency.Enum
{
    internal enum CommandType
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
        Premium,
        Publish,
        Startup,
        Cycyling,
        Property,
        StartupM,
        StartupP,
        LockScreenApply,
        LockScreenRemove,
        LockScreenFeature,
        Official,
        Watchdog,
        Interface,
        PropertyA,
        Reportdog,
        Scheduler,
        Repository,
        Versioning,
        Discussions,
        RestartLive,
        Backgroundog,
        QuotingGoogle,
        QuotingTranslate
    }

    internal enum ArgumentCommandType
    {
        OtherSetting,
        DonateSetting,
        SystemSetting,
        GeneralSetting,
        PersonalSetting,
        WallpaperSetting,
        PerformanceSetting
    }

    internal enum SchedulerCommandType
    {
        Create,
        Enable,
        Delete,
        Disable
    }
}