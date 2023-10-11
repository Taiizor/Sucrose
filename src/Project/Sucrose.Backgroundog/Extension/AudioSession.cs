using NPSMLib;
using System.IO;
using SBET = Sucrose.Backgroundog.Extension.Thumbnail;
using SBMI = Sucrose.Backgroundog.Manage.Internal;

namespace Sucrose.Backgroundog.Extension
{
    internal static class AudioSession
    {
        private static void SetupEvents()
        {
            if (SBMI.PlayingSession != null)
            {
                SBMI.DataSource = SBMI.PlayingSession.ActivateMediaPlaybackDataSource();
                SBMI.DataSource.MediaPlaybackDataChanged += (s, e) => SetCurrentSession();
            }
        }

        public static void SessionListChanged()
        {
            if (SBMI.SessionManager != null)
            {
                SBMI.PlayingSession = SBMI.SessionManager.CurrentSession;
                SetupEvents();
                SetCurrentSession();
            }
        }

        private static void SetCurrentSession()
        {
            if (SBMI.PlayingSession != null)
            {
                lock (SBMI.LockObject)
                {
                    MediaObjectInfo MediaDetails = SBMI.DataSource.GetMediaObjectInfo();
                    MediaPlaybackInfo MediaPlaybackInfo = SBMI.DataSource.GetMediaPlaybackInfo();
                    MediaTimelineProperties MediaTimeline = SBMI.DataSource.GetMediaTimelineProperties();

                    string ThumbnailString = null;

                    using (Stream Thumbnail = SBMI.DataSource.GetThumbnailStream())
                    {
                        if (Thumbnail != null)
                        {
                            ThumbnailString = SBET.Create(Thumbnail);
                            Thumbnail.Flush();
                            Thumbnail.Dispose();
                        }
                    }

                    SBMI.AudioData.State = true;
                    SBMI.AudioData.Title = MediaDetails.Title;
                    SBMI.AudioData.Artist = MediaDetails.Artist;
                    SBMI.AudioData.Subtitle = MediaDetails.Subtitle;
                    SBMI.AudioData.AlbumTitle = MediaDetails.AlbumTitle;
                    SBMI.AudioData.AlbumArtist = MediaDetails.AlbumArtist;
                    SBMI.AudioData.TrackNumber = MediaDetails.TrackNumber;
                    SBMI.AudioData.AlbumTrackCount = MediaDetails.AlbumTrackCount;
                    SBMI.AudioData.MediaType = MediaPlaybackDataSource.MediaSchemaToMediaPlaybackMode(MediaDetails.MediaClassPrimaryID);

                    //SBMI.AudioData.PID = SBMI.PlayingSession?.PID;
                    //SBMI.AudioData.Hwnd = SBMI.PlayingSession?.Hwnd;
                    SBMI.AudioData.SourceAppId = SBMI.PlayingSession?.SourceAppId;
                    //SBMI.AudioData.SourceDeviceId = SBMI.PlayingSession?.SourceDeviceId;
                    //SBMI.AudioData.RenderDeviceId = SBMI.PlayingSession?.RenderDeviceId;

                    SBMI.AudioData.RepeatMode = MediaPlaybackInfo.RepeatMode;
                    SBMI.AudioData.PropsValid = MediaPlaybackInfo.PropsValid;
                    SBMI.AudioData.PlaybackRate = MediaPlaybackInfo.PlaybackRate;
                    SBMI.AudioData.PlaybackMode = MediaPlaybackInfo.PlaybackMode;
                    //SBMI.AudioData.PlaybackCaps = MediaPlaybackInfo.PlaybackCaps;
                    SBMI.AudioData.PlaybackState = MediaPlaybackInfo.PlaybackState;
                    SBMI.AudioData.ShuffleEnabled = MediaPlaybackInfo.ShuffleEnabled;
                    SBMI.AudioData.LastPlayingFileTime = MediaPlaybackInfo.LastPlayingFileTime;

                    SBMI.AudioData.EndTime = MediaTimeline.EndTime;
                    SBMI.AudioData.Position = MediaTimeline.Position;
                    SBMI.AudioData.StartTime = MediaTimeline.StartTime;
                    SBMI.AudioData.MinSeekTime = MediaTimeline.MinSeekTime;
                    SBMI.AudioData.MaxSeekTime = MediaTimeline.MaxSeekTime;
                    SBMI.AudioData.PositionSetFileTime = MediaTimeline.PositionSetFileTime;

                    SBMI.AudioData.ThumbnailString = ThumbnailString;

                    if (string.IsNullOrEmpty(ThumbnailString))
                    {
                        SBMI.AudioData.ThumbnailAddress = ThumbnailString;
                    }
                    else
                    {
                        SBMI.AudioData.ThumbnailAddress = "data:image/png;base64," + ThumbnailString;
                    }
                }
            }
            else
            {
                lock (SBMI.LockObject)
                {
                    SBMI.AudioData.State = false;
                }
            }
        }
    }
}