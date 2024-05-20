namespace Sucrose.Mpv.NET.Player
{
    public enum KeepOpen
    {
        /// <summary>
        /// When the current media ends, play the next media or stop.
        /// </summary>
        No,

        /// <summary>
        /// Do not unload media when it reaches the end and it is the last entry in the playlist.
        /// </summary>
        Yes,

        /// <summary>
        /// Similar to "Yes" but it will not advance to the next entry in the playlist.
        /// </summary>
        Always
    }
}