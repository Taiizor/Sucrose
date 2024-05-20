using Sucrose.Mpv.NET.API;
using System.Diagnostics;
using System.Globalization;

namespace Sucrose.Mpv.NET.Player
{
    /// <summary>
    /// User control containing an mpv player.
    /// </summary>
    public partial class MpvPlayer : IMpvPlayer, IDisposable
    {
        /// <summary>
        /// An instance of the underlying mpv API. Do not touch unless you know what you're doing.
        /// </summary>
        public API.Mpv API { get; private set; }

        /// <summary>
        /// Absolute or relative (to your executable) path to the libmpv DLL.
        /// </summary>
        public string LibMpvPath { get; private set; }

        /// <summary>
        /// Title of the loaded media. If the title is not available, this falls back to the path.
        /// </summary>
        public string MediaTitle { get; private set; }

        /// <summary>
        /// Indicates whether media will automatically play when loaded.
        /// </summary>
        public bool AutoPlay { get; set; }

        /// <summary>
        /// True when media is loaded and ready for playback.
        /// </summary>
        public bool IsMediaLoaded { get; private set; }

        /// <summary>
        /// True if media is playing.
        /// </summary>
        public bool IsPlaying { get; private set; }

        /// <summary>
        /// True if media is paused waiting for the cache to be filled. This indicates whether the media is "buffering"
        /// </summary>
        public bool IsPausedForCache
        {
            get
            {
                string yesNoString;
                lock (mpvLock)
                {
                    yesNoString = API.GetPropertyString("paused-for-cache");
                }

                return MpvPlayerHelper.YesNoToBool(yesNoString);
            }
        }

        /// <summary>
        /// In seconds, the approximate duration of video buffered.
        /// </summary>
        public double CacheDuration
        {
            get
            {
                lock (mpvLock)
                {
                    return API.GetPropertyDouble("cache-duration");
                }
            }
        }

        /// <summary>
        /// Set the logging level for mpv.
        /// </summary>
        public MpvLogLevel LogLevel
        {
            set
            {
                lock (mpvLock)
                {
                    API.RequestLogMessages(value);
                }

                logLevel = value;
            }
            get => logLevel;
        }

        /// <summary>
        /// The desired video quality to retrieve when loading streams from video sites.
        /// </summary>
        public YouTubeDlVideoQuality YouTubeDlVideoQuality
        {
            get => ytdlVideoQuality;
            set
            {
                string formatString = YouTubeDlHelperQuality.GetFormatStringForVideoQuality(value);

                lock (mpvLock)
                {
                    API.SetPropertyString("ytdl-format", formatString);
                }

                ytdlVideoQuality = value;
            }
        }

        /// <summary>
        /// Number of entries in the playlist.
        /// </summary>
        public int PlaylistEntryCount
        {
            get
            {
                lock (mpvLock)
                {
                    return (int)API.GetPropertyLong("playlist-count");
                }
            }
        }

        /// <summary>
        /// Index of the current entry in the playlist. (zero based)
        /// </summary>
        public int PlaylistIndex
        {
            get
            {
                lock (mpvLock)
                {
                    return (int)API.GetPropertyLong("playlist-pos");
                }
            }
            set
            {
                lock (mpvLock)
                {
                    API.SetPropertyLong("playlist-pos", value);
                }
            }
        }

        /// <summary>
        /// If true, media will not be unloaded when playback finishes.
        /// Warning: The MediaUnloaded event will not be raised, use EndReached
        /// property or MediaFinished event to detect end of playback instead.
        /// </summary>
        public KeepOpen KeepOpen
        {
            get
            {
                string stringValue;
                lock (mpvLock)
                {
                    stringValue = API.GetPropertyString("keep-open");
                }

                return KeepOpenHelper.FromString(stringValue);
            }
            set
            {
                string stringValue = KeepOpenHelper.ToString(value);

                lock (mpvLock)
                {
                    API.SetPropertyString("keep-open", stringValue);
                }
            }
        }

        /// <summary>
        /// Determines whether media will loop.
        /// </summary>
        public bool Loop
        {
            get
            {
                string stringValue;
                lock (mpvLock)
                {
                    stringValue = API.GetPropertyString("loop");
                }

                return MpvPlayerHelper.YesNoToBool(stringValue);
            }
            set
            {
                string stringValue = MpvPlayerHelper.BoolToYesNo(value);

                lock (mpvLock)
                {
                    API.SetPropertyString("loop", stringValue);
                }
            }
        }

        /// <summary>
        /// Determines whether playlist will loop.
        /// </summary>
        public bool LoopPlaylist
        {
            get
            {
                string stringValue;

                lock (mpvLock)
                {
                    stringValue = API.GetPropertyString("loop-playlist");
                }

                return stringValue != "no";
            }
            set
            {
                string stringValue = value ? "inf" : "no";

                lock (mpvLock)
                {
                    API.SetPropertyString("loop-playlist", stringValue);
                }
            }
        }

        /// <summary>
        /// Indicates whether media has reached end of playback.
        /// </summary>
        public bool EndReached
        {
            get
            {
                string stringValue;
                lock (mpvLock)
                {
                    stringValue = API.GetPropertyString("eof-reached");
                }

                return MpvPlayerHelper.YesNoToBool(stringValue);
            }
        }

        /// <summary>
        /// Duration of the media file. (As indicated by metadata)
        /// </summary>
        public TimeSpan Duration
        {
            get
            {
                if (!IsMediaLoaded)
                {
                    return TimeSpan.Zero;
                }

                double duration;
                lock (mpvLock)
                {
                    duration = API.GetPropertyDouble("duration");
                }

                return TimeSpan.FromSeconds(duration);
            }
        }

        /// <summary>
        /// Current position of playback in the media player.
        /// </summary>
        public TimeSpan Position
        {
            get
            {
                if (!IsMediaLoaded)
                {
                    return TimeSpan.Zero;
                }

                double position;
                lock (mpvLock)
                {
                    position = API.GetPropertyDouble("time-pos");
                }

                return TimeSpan.FromSeconds(position);
            }
            set
            {
                GuardAgainstNotLoaded();

                if (value < TimeSpan.Zero || value > Duration)
                {
                    throw new ArgumentOutOfRangeException("Desired position is out of range of the duration or less than zero.");
                }

                double totalSeconds = value.TotalSeconds;

                string totalSecondsString = totalSeconds.ToString(CultureInfo.InvariantCulture);

                lock (mpvLock)
                {
                    API.Command("seek", totalSecondsString, "absolute");
                }
            }
        }

        /// <summary>
        /// Time left of playback in the media file.
        /// </summary>
        public TimeSpan Remaining
        {
            get
            {
                if (!IsMediaLoaded)
                {
                    return TimeSpan.Zero;
                }

                double remaining;
                lock (mpvLock)
                {
                    remaining = API.GetPropertyDouble("time-remaining");
                }

                return TimeSpan.FromSeconds(remaining);
            }
        }

        /// <summary>
        /// Volume of the current media. Ranging from 0 to 100 inclusive.
        /// </summary>
        public int Volume
        {
            get
            {
                lock (mpvLock)
                {
                    return (int)API.GetPropertyDouble("volume");
                }
            }
            set
            {
                if (value is < 0 or > 100)
                {
                    throw new ArgumentOutOfRangeException("Volume should be between 0 and 100 (inclusive).");
                }

                lock (mpvLock)
                {
                    API.SetPropertyDouble("volume", value);
                }
            }
        }

        /// <summary>
        /// Playback speed of the current media. Ranging from 0.01 to 100 inclusive.
        /// For example, 0.5 is half the speed, 1 is normal speed and 2 is double the speed.
        /// </summary>
        public double Speed
        {
            get
            {
                lock (mpvLock)
                {
                    return API.GetPropertyDouble("speed");
                }
            }
            set
            {
                if (value is < 0.01 or > 100)
                {
                    throw new ArgumentOutOfRangeException("Speed should be between 0.01 and 100 (inclusive).");
                }

                lock (mpvLock)
                {
                    API.SetPropertyDouble("speed", value);
                }
            }
        }

        /// <summary>
        /// List of file names representing the currently loaded playlist, ordered by their 0-based indexes
        /// </summary>
        public IReadOnlyList<string> CurrentPlaylist
        {
            get
            {
                lock (mpvLock)
                {
                    return Enumerable.Range(0, PlaylistEntryCount).Select(i => API.GetPropertyString($"playlist/{i}/filename")).ToArray();
                }
            }
        }

        /// <summary>
        /// Invoked when media is resumed.
        /// </summary>
        public event EventHandler MediaResumed;

        /// <summary>
        /// Invoked when media is paused.
        /// </summary>
        public event EventHandler MediaPaused;

        /// <summary>
        /// Invoked when media started buffering.
        /// </summary>
        public event EventHandler MediaStartedBuffering;

        /// <summary>
        /// Invoked when media ended buffering.
        /// </summary>
        public event EventHandler MediaEndedBuffering;

        /// <summary>
        /// Invoked when media is loaded.
        /// </summary>
        public event EventHandler MediaLoaded;

        /// <summary>
        /// Invoked when media is unloaded.
        /// </summary>
        public event EventHandler MediaUnloaded;

        /// <summary>
        /// Invoked when media finished playback.
        /// </summary>
        public event EventHandler MediaFinished;

        /// <summary>
        /// Invoked when an error occurs with the media. (e.g. failed to load)
        /// </summary>
        public event EventHandler MediaError;

        /// <summary>
        /// Invoked when started seeking.
        /// </summary>
        public event EventHandler MediaStartedSeeking;

        /// <summary>
        /// Invoked when seeking has ended and media is ready to be played.
        /// </summary>
        public event EventHandler MediaEndedSeeking;

        /// <summary>
        /// Invoked when the Position ("time-pos" in mpv) property is changed. Event arguments contain the new position.
        /// </summary>
        public event EventHandler<MpvPlayerPositionChangedEventArgs> PositionChanged
        {
            add
            {
                lock (mpvLock)
                {
                    API.ObserveProperty("time-pos", MpvFormat.Double, timePosUserData);
                }

                positionChanged += value;
            }
            remove
            {
                lock (mpvLock)
                {
                    API.UnobserveProperty(timePosUserData);
                }

                positionChanged -= null;
            }
        }

        private readonly IntPtr hwnd;

        private EventHandler<MpvPlayerPositionChangedEventArgs> positionChanged;

        private MpvLogLevel logLevel = MpvLogLevel.None;

        private YouTubeDlVideoQuality ytdlVideoQuality;
        private bool isYouTubeDlEnabled = false;

        // External seeking is when SeekAsync is called.
        private bool isExternalSeeking = false;
        private bool isSeeking = false;

        private TaskCompletionSource<object> seekCompletionSource;

        private const int timePosUserData = 10;
        private const int pauseUserData = 20;
        private const int eofReachedUserData = 30;
        private const int pausedForCacheUserData = 40;

        private readonly string[] possibleLibMpvPaths = new string[]
        {
            "mpv-2.dll",
            @"lib\mpv-2.dll"
        };

        private readonly string[] possibleYtdlHookPaths = new string[]
        {
            "ytdl_hook.lua",
            "ytdl.lua",
            @"lib\ytdl_hook.lua",
            @"lib\ytdl.lua"
        };

        private readonly object mpvLock = new();

        /// <summary>
        /// Creates an instance of MpvPlayer. This will let mpv create it's own window and
        /// the libmpv will be searched for in default locations.
        /// </summary>
        public MpvPlayer() : this(IntPtr.Zero)
        {
        }

        /// <summary>
        /// Creates an instance of MpvPlayer. This will let mpv create it's own window.
        /// </summary>
        /// <param name="libMpvPath">Relative or absolute path to the libmpv DLL.</param>
        public MpvPlayer(string libMpvPath) : this(IntPtr.Zero, libMpvPath)
        {
        }

        /// <summary>
        /// Creates an instance of MpvPlayer.
        /// </summary>
        /// <param name="hwnd">The windows handle that will host Mpv, such as one created with a WindowsFormsHost in WPF.</param>
        public MpvPlayer(IntPtr hwnd)
        {
            this.hwnd = hwnd;

            Initialise();
        }

        /// <summary>
        /// Creates an instance of MpvPlayer using a specific libmpv DLL.
        /// </summary>
        /// <param name="hwnd">The windows handle that will host Mpv, such as one created with a WindowsFormsHost in WPF.</param>
        /// <param name="libMpvPath">Relative or absolute path to the libmpv DLL.</param>
        public MpvPlayer(IntPtr hwnd, string libMpvPath)
        {
            this.LibMpvPath = libMpvPath;
            this.hwnd = hwnd;

            Initialise();
        }

        private void Initialise()
        {
            // Initialise the API.
            if (!string.IsNullOrEmpty(LibMpvPath))
            {
                InitialiseMpv(LibMpvPath);
            }
            else
            {
                string foundPath = possibleLibMpvPaths.FirstOrDefault(File.Exists);
                if (foundPath != null)
                {
                    InitialiseMpv(foundPath);
                }
                else
                {
                    throw new MpvPlayerException("Failed to find libmpv. Check your path.");
                }
            }

            // Set defaults.
            Volume = 50;
            YouTubeDlVideoQuality = YouTubeDlVideoQuality.Highest;

            // Set the host of the mpv player.
            if (hwnd != IntPtr.Zero)
            {
                SetMpvHost(hwnd);
            }
        }

        private void InitialiseMpv(string libMpvPath)
        {
            API = new API.Mpv(libMpvPath);

            API.LogMessage += MpvOnLogMessage;

            API.PlaybackRestart += MpvOnPlaybackRestart;
            API.Seek += MpvOnSeek;

            API.FileLoaded += MpvOnFileLoaded;
            API.EndFile += MpvOnEndFile;

            API.PropertyChange += MpvOnPropertyChange;

            API.ObserveProperty("pause", MpvFormat.String, pauseUserData);
            API.ObserveProperty("eof-reached", MpvFormat.String, eofReachedUserData);
            API.ObserveProperty("paused-for-cache", MpvFormat.String, pausedForCacheUserData);
        }

        private void SetMpvHost(IntPtr hwnd)
        {
            long playerHostPtrLong = hwnd.ToInt64();
            API.SetPropertyLong("wid", playerHostPtrLong);
        }

        /// <summary>
        /// Loads the file at the path into mpv. If called while media is playing, the specified media
        /// will be appended to the playlist.
        /// If yt-dlp is enabled, this method can be used to load videos from video sites.
        /// </summary>
        /// <param name="path">Path or URL to media source.</param>
        /// <param name="force">If true, will force load the media replacing any currently playing media.</param>
        public void Load(string path, bool force = false)
        {
            Guard.AgainstNullOrEmptyOrWhiteSpaceString(path, nameof(path));

            lock (mpvLock)
            {
                API.SetPropertyString("pause", AutoPlay ? "no" : "yes");

                LoadMethod loadMethod = LoadMethod.Replace;

                // If there is media already playing, we append
                // the desired video onto the playlist.
                // (Unless force is true.)
                if (IsPlaying && !force)
                {
                    loadMethod = LoadMethod.Append;
                }

                string loadMethodString = LoadMethodHelper.ToString(loadMethod);
                API.Command("loadfile", path, loadMethodString);
            }
        }

        /// <summary>
        /// Loads a collection of file paths as a playlist into mpv. If called while media is playing,
        /// the specified media collection will be appended to the playlist. If yt-dlp is enabled,
        /// this method can be used to load videos from video sites.
        /// </summary>
        /// <param name="paths">Paths or URLs to media sources</param>
        /// <param name="force">If true, will force load the media replacing any currently playing media.</param>
        public void LoadPlaylist(IEnumerable<string> paths, bool force = false)
        {
            Guard.AgainstNull(paths);

            lock (mpvLock)
            {
                API.SetPropertyString("pause", AutoPlay ? "no" : "yes");

                bool first = true;

                foreach (string path in paths)
                {
                    LoadMethod loadMethod = LoadMethod.Append;

                    if (first && (!force || !IsPlaying))
                    {
                        loadMethod = LoadMethod.Replace;
                    }

                    string loadMethodString = LoadMethodHelper.ToString(loadMethod);

                    API.Command("loadfile", path, loadMethodString);

                    first = false;
                }
            }
        }

        /// <summary>
        /// Seek using the specified position. Seeking absolutely by default.
        /// </summary>
        /// <param name="position">Position in seconds.</param>
        /// <param name="relative"></param>
        /// <returns>Task that will complete when seeking completes.</returns>
        public Task SeekAsync(double position, bool relative = false)
        {
            return SeekAsync(TimeSpan.FromSeconds(position), relative);
        }

        /// <summary>
        /// Seek using the specified position. Seeking absolutely by default.
        /// </summary>
        /// <param name="relative">If true, the given position is added to the current position.</param>
        /// <returns>Task that will complete when seeking completes.</returns>
        public Task SeekAsync(TimeSpan position, bool relative = false)
        {
            seekCompletionSource = new TaskCompletionSource<object>();

            isExternalSeeking = true;

            if (relative)
            {
                Position += position;
            }
            else
            {
                Position = position;
            }

            return seekCompletionSource.Task;
        }

        /// <summary>
        /// Resume playback.
        /// </summary>
        public void Resume()
        {
            lock (mpvLock)
            {
                API.SetPropertyString("pause", "no");
            }

            IsPlaying = true;
        }

        /// <summary>
        /// Pause playback.
        /// </summary>
        public void Pause()
        {
            lock (mpvLock)
            {
                API.SetPropertyString("pause", "yes");
            }

            IsPlaying = false;
        }

        /// <summary>
        /// Stop playback and unload the media file.
        /// </summary>
        public void Stop()
        {
            lock (mpvLock)
            {
                API.Command("stop");
            }

            IsMediaLoaded = false;
            IsPlaying = false;
        }

        /// <summary>
        /// Goes to the start of the media file and resumes playback.
        /// </summary>
        public async Task RestartAsync()
        {
            await SeekAsync(TimeSpan.Zero);

            Resume();
        }

        /// <summary>
        /// Load an audio track. Must be called after Load.
        /// </summary>
        /// <param name="path">Path to the audio track.</param>
        public void AddAudio(string path)
        {
            lock (mpvLock)
            {
                API.Command("audio-add", path);
            }
        }

        /// <summary>
        /// Go to the next entry in the playlist.
        /// </summary>
        /// <returns>True if successful, false if not. False indicates that there are no entries after the current entry.</returns>
        public bool PlaylistNext()
        {
            try
            {
                lock (mpvLock)
                {
                    API.Command("playlist-next");
                }

                return true;
            }
            catch (MpvAPIException exception)
            {
                return HandleCommandMpvAPIException(exception);
            }
        }

        /// <summary>
        /// Go to the previous entry in the playlist.
        /// </summary>
        /// <returns>True if successful, false if not. False indicates that there are no entries before the current entry.</returns>
        public bool PlaylistPrevious()
        {
            try
            {
                lock (mpvLock)
                {
                    API.Command("playlist-prev");
                }

                return true;
            }
            catch (MpvAPIException exception)
            {
                return HandleCommandMpvAPIException(exception);
            }
        }

        /// <summary>
        /// Remove the current entry from the playlist.
        /// </summary>
        /// <returns>True if removed, false if not.</returns>
        public bool PlaylistRemove()
        {
            try
            {
                lock (mpvLock)
                {
                    API.Command("playlist-remove", "current");
                }

                return true;
            }
            catch (MpvAPIException exception)
            {
                return HandleCommandMpvAPIException(exception);
            }
        }

        /// <summary>
        /// Removes a specific entry in the playlist, indicated by an index.
        /// </summary>
        /// <param name="index">Zero based index to an entry in the playlist.</param>
        /// <returns>True if removed, false if not.</returns>
        public bool PlaylistRemove(int index)
        {
            string indexString = index.ToString();

            try
            {
                lock (mpvLock)
                {
                    API.Command("playlist-remove", indexString);
                }

                return true;
            }
            catch (MpvAPIException exception)
            {
                return HandleCommandMpvAPIException(exception);
            }
        }

        /// <summary>
        /// Moves the playlist entry at oldIndex to newIndex. This does not swap the entries.
        /// </summary>
        /// <param name="oldIndex">Index of the entry you want to move.</param>
        /// <param name="newIndex">Index of where you want to move the entry.</param>
        /// <returns>True if moved, false if not.</returns>
        public bool PlaylistMove(int oldIndex, int newIndex)
        {
            string oldIndexString = oldIndex.ToString();
            string newIndexString = newIndex.ToString();

            try
            {
                lock (mpvLock)
                {
                    API.Command("playlist-move", oldIndexString, newIndexString);
                }

                return true;
            }
            catch (MpvAPIException exception)
            {
                return HandleCommandMpvAPIException(exception);
            }
        }

        /// <summary>
        /// Clear the playlist of all entries,
        /// </summary>
        public void PlaylistClear()
        {
            lock (mpvLock)
            {
                API.Command("playlist-clear");
            }
        }

        /// <summary>
        /// Enable yt-dlp functionality in mpv. This methods attempts to load the ytdl hook script from:
        /// "ytdl_hook.lua",
        /// "ytdl.lua",
        /// "lib\ytdl_hook.lua" and
        /// "lib\ytdl.lua"
        /// </summary>
        /// <exception cref="FileNotFoundException">Thrown when none of the ytdl hook scripts are found.</exception>
        public void EnableYouTubeDl()
        {
            if (isYouTubeDlEnabled)
            {
                return;
            }

            string foundPath = possibleYtdlHookPaths.FirstOrDefault(File.Exists);
            if (foundPath != default)
            {
                EnableYouTubeDl(foundPath);
            }
            else
            {
                throw new FileNotFoundException("Cannot find ytdl hook script.");
            }
        }

        /// <summary>
        /// Enable yt-dlp functionality in mpv.
        /// </summary>
        /// <param name="ytdlHookScriptPath">Relative or absolute path to the ytdl hook script. (usually called "ytdl_hook.lua")</param>
        /// <exception cref="FileNotFoundException">Throw when the ytdl hook script is not found.</exception>
        public void EnableYouTubeDl(string ytdlHookScriptPath)
        {
            if (isYouTubeDlEnabled)
            {
                return;
            }

            Guard.AgainstNullOrEmptyOrWhiteSpaceString(ytdlHookScriptPath, nameof(ytdlHookScriptPath));

            if (!File.Exists(ytdlHookScriptPath))
            {
                throw new FileNotFoundException("Cannot find ytdl hook script.");
            }

            lock (mpvLock)
            {
                API.Command("load-script", ytdlHookScriptPath);
            }

            isYouTubeDlEnabled = true;
        }

        /// <summary>
        /// Instruct mpv to load a config file.
        /// </summary>
        /// <param name="configFilePath">Absolute path to config file.</param>
        public void LoadConfig(string configFilePath)
        {
            lock (mpvLock)
            {
                API.LoadConfigFile(configFilePath);
            }
        }

        /// <summary>
        /// Move to the next frame. Does not work with audio-only.
        /// </summary>
        public void NextFrame()
        {
            lock (mpvLock)
            {
                API.Command("frame-step");
            }
        }

        /// <summary>
        /// Move to the previous frame. Note that this is slow since it's trying to be precise, not fast.
        /// </summary>
        public void PreviousFrame()
        {
            lock (mpvLock)
            {
                API.Command("frame-back-step");
            }
        }

        private void MpvOnPlaybackRestart(object sender, EventArgs e)
        {
            if (isSeeking)
            {
                if (isExternalSeeking)
                {
                    seekCompletionSource.SetResult(null);
                    isExternalSeeking = false;
                }

                MediaEndedSeeking?.Invoke(this, EventArgs.Empty);
                isSeeking = false;
            }
        }

        private void MpvOnSeek(object sender, EventArgs e)
        {
            isSeeking = true;
            MediaStartedSeeking?.Invoke(this, EventArgs.Empty);
        }

        private void MpvOnFileLoaded(object sender, EventArgs e)
        {
            IsMediaLoaded = true;

            IsPlaying = AutoPlay;

            MediaTitle = API.GetPropertyString("media-title");

            MediaLoaded?.Invoke(this, EventArgs.Empty);
        }

        private void MpvOnEndFile(object sender, MpvEndFileEventArgs e)
        {
            IsMediaLoaded = false;
            isSeeking = false;

            MpvEventEndFile eventEndFile = e.EventEndFile;

            if (eventEndFile.Reason == MpvEndFileReason.EndOfFile)
            {
                MediaFinished?.Invoke(this, EventArgs.Empty);
            }

            switch (eventEndFile.Reason)
            {
                case MpvEndFileReason.Stop:
                case MpvEndFileReason.Quit:
                case MpvEndFileReason.Redirect:
                case MpvEndFileReason.EndOfFile:
                    MediaUnloaded?.Invoke(this, EventArgs.Empty);
                    break;
                case MpvEndFileReason.Error:
                    MediaError?.Invoke(this, EventArgs.Empty);
                    break;
            }
        }

        private void MpvOnLogMessage(object sender, MpvLogMessageEventArgs e)
        {
            MpvLogMessage message = e.Message;

            string prefix = message.Prefix;
            string text = message.Text;

            string output = $"[{prefix}] {text}";

#if DEBUG
            Debug.Write(output);
#else
            Trace.Write(output);
#endif
        }

        private void MpvOnPropertyChange(object sender, MpvPropertyChangeEventArgs e)
        {
            MpvEventProperty eventProperty = e.EventProperty;

            switch (e.ReplyUserData)
            {
                case timePosUserData:
                    double newPosition = eventProperty.DataDouble;

                    InvokePositionChanged(newPosition);
                    break;
                case pauseUserData:
                    string pausedYesNoString = eventProperty.DataString;
                    bool paused = MpvPlayerHelper.YesNoToBool(pausedYesNoString);

                    if (paused)
                    {
                        MediaPaused?.Invoke(this, EventArgs.Empty);
                    }
                    else
                    {
                        MediaResumed?.Invoke(this, EventArgs.Empty);
                    }

                    break;
                case eofReachedUserData:
                    string eofReachedYesNoString = eventProperty.DataString;
                    bool eofReached = MpvPlayerHelper.YesNoToBool(eofReachedYesNoString);

                    if (eofReached)
                    {
                        MediaFinished?.Invoke(this, EventArgs.Empty);
                    }

                    break;
                case pausedForCacheUserData:
                    string pausedForCacheYesNoString = eventProperty.DataString;

                    bool pausedForCache = MpvPlayerHelper.YesNoToBool(pausedForCacheYesNoString);
                    if (pausedForCache)
                    {
                        MediaStartedBuffering?.Invoke(this, EventArgs.Empty);
                    }
                    else
                    {
                        MediaEndedBuffering?.Invoke(this, EventArgs.Empty);
                    }

                    break;
            }
        }

        private void InvokePositionChanged(double newPosition)
        {
            MpvPlayerPositionChangedEventArgs eventArgs = new(newPosition);
            positionChanged?.Invoke(this, eventArgs);
        }

        private void GuardAgainstNotLoaded()
        {
            if (!IsMediaLoaded)
            {
                throw new InvalidOperationException("Operation could not be completed because no media file has been loaded.");
            }
        }

        private static bool HandleCommandMpvAPIException(MpvAPIException exception)
        {
            if (exception.Error == MpvError.Command)
            {
                return false;
            }
            else
            {
                throw exception;
            }
        }

        public void Dispose()
        {
            API.Dispose();
        }
    }
}