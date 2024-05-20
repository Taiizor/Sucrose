namespace Sucrose.Mpv.NET.API
{
    public interface IMpvFunctions
    {
        MpvClientAPIVersion ClientAPIVersion { get; }
        MpvClientId ClientId { get; }
        MpvErrorString ErrorString { get; }
        MpvFree Free { get; }
        MpvClientName ClientName { get; }
        MpvCreate Create { get; }
        MpvInitialise Initialise { get; }
        MpvDestroy Destroy { get; }
        MpvTerminateDestroy TerminateDestroy { get; }
        MpvCreateClient CreateClient { get; }
        MpvCreateWeakClient CreateWeakClient { get; }
        MpvLoadConfigFile LoadConfigFile { get; }
        MpvGetTimeUs GetTimeUs { get; }
        MpvSetOption SetOption { get; }
        MpvSetOptionString SetOptionString { get; }
        MpvCommand Command { get; }
        MpvCommandString CommandString { get; }
        MpvCommandAsync CommandAsync { get; }
        MpvAbortAsyncCommand AbortAsyncCommand { get; }
        MpvSetProperty SetProperty { get; }
        MpvSetPropertyString SetPropertyString { get; }
        MpvSetPropertyAsync SetPropertyAsync { get; }
        MpvGetProperty GetProperty { get; }
        MpvGetPropertyString GetPropertyString { get; }
        MpvGetPropertyOSDString GetPropertyOSDString { get; }
        MpvGetPropertyAsync GetPropertyAsync { get; }
        MpvObserveProperty ObserveProperty { get; }
        MpvUnobserveProperty UnobserveProperty { get; }
        MpvEventName EventName { get; }
        MpvRequestEvent RequestEvent { get; }
        MpvRequestLogMessages RequestLogMessages { get; }
        MpvWaitEvent WaitEvent { get; }
        MpvWakeup Wakeup { get; }
        MpvSetWakeupCallback SetWakeupCallback { get; }
        MpvWaitAsyncRequests WaitAsyncRequests { get; }
        MpvHookAdd HookAdd { get; }
        MpvHookContinue HookContinue { get; }

        MpvGetPropertyDouble GetPropertyDouble { get; }
        MpvGetPropertyLong GetPropertyLong { get; }
    }
}