using Sucrose.Player.ME.Manage;
using System.Diagnostics;
using System.Windows;

namespace Sucrose.Player.ME.Event
{
    internal static class Handler
    {
        public static void MediaPlayerOpened(object sender, RoutedEventArgs e)
        {
            Debug.WriteLine("Media playback opened: success");
        }

        public static void MediaPlayerFailed(object? sender, ExceptionRoutedEventArgs e)
        {
            Debug.WriteLine("Media playback failure: " + e.ErrorException.Message);
        }

        public static void MediaPlayerEnded(object sender, EventArgs e)
        {
            Internal.MediaPlayer.Position = TimeSpan.Zero;
            Internal.MediaPlayer.Play();
        }
    }
}