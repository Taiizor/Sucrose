namespace Sucrose.Mpv.NET.API
{
    public enum MpvEventID
    {
        None = 0,
        Shutdown = 1,
        LogMessage = 2,
        GetPropertyReply = 3,
        SetPropertyReply = 4,
        CommandReply = 5,
        StartFile = 6,
        EndFile = 7,
        FileLoaded = 8,
        TracksChanged = 9,
        TrackSwitched = 10,
        Pause = 12,
        Unpause = 13,
        ScriptInputDispatch = 15,
        ClientMessage = 16,
        VideoReconfig = 17,
        AudioReconfig = 18,
        MetadataUpdate = 19,
        Seek = 20,
        PlaybackRestart = 21,
        PropertyChange = 22,
        ChapterChange = 23,
        QueueOverflow = 24,
        EventHook = 25
    }
}