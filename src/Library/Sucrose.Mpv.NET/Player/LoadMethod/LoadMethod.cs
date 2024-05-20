namespace Sucrose.Mpv.NET.Player
{
    public enum LoadMethod
    {
        /// <summary>
        /// Stop playback of current media and start new one.
        /// </summary>
        Replace,

        /// <summary>
        /// Append media to playlist.
        /// </summary>
        Append,

        /// <summary>
        /// Append media to playlist and play if nothing else is playing.
        /// </summary>
        AppendPlay
    }
}