using Sucrose.Mpv.NET.API;

namespace Sucrose.Mpv.NET.Player
{
    public interface IMpvPlayer
    {
        API.Mpv API { get; }

        string LibMpvPath { get; }

        string MediaTitle { get; }

        bool AutoPlay { get; set; }

        bool IsMediaLoaded { get; }

        bool IsPlaying { get; }

        bool IsPausedForCache { get; }

        double CacheDuration { get; }

        MpvLogLevel LogLevel { get; set; }

        YouTubeDlVideoQuality YouTubeDlVideoQuality { get; set; }

        int PlaylistEntryCount { get; }

        int PlaylistIndex { get; }

        KeepOpen KeepOpen { get; set; }

        bool Loop { get; set; }

        bool LoopPlaylist { get; set; }

        bool EndReached { get; }

        TimeSpan Duration { get; }

        TimeSpan Position { get; }

        TimeSpan Remaining { get; }

        int Volume { get; set; }

        double Speed { get; set; }

        IReadOnlyList<string> CurrentPlaylist { get; }

        event EventHandler MediaResumed;

        event EventHandler MediaPaused;

        event EventHandler MediaStartedBuffering;

        event EventHandler MediaEndedBuffering;

        event EventHandler MediaLoaded;

        event EventHandler MediaUnloaded;

        event EventHandler MediaFinished;

        event EventHandler MediaError;

        event EventHandler MediaStartedSeeking;

        event EventHandler MediaEndedSeeking;

        event EventHandler<MpvPlayerPositionChangedEventArgs> PositionChanged;

        void Load(string path, bool force = false);

        void LoadPlaylist(IEnumerable<string> paths, bool force = false);

        Task SeekAsync(double position, bool relative = false);

        Task SeekAsync(TimeSpan position, bool relative = false);

        void Resume();

        void Pause();

        void Stop();

        Task RestartAsync();

        void AddAudio(string path);

        bool PlaylistNext();

        bool PlaylistPrevious();

        bool PlaylistRemove();

        bool PlaylistRemove(int index);

        bool PlaylistMove(int oldIndex, int newIndex);

        void PlaylistClear();

        void EnableYouTubeDl();

        void EnableYouTubeDl(string ytdlHookScriptPath);

        void NextFrame();

        void PreviousFrame();
    }
}