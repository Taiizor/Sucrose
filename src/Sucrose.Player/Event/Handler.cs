using Microsoft.Web.WebView2.Core;
using Skylark.Wing.Helper;
using Sucrose.Player.Manage;
using System.Diagnostics;
using System.Windows;

namespace Sucrose.Player.Event
{
    internal static class Handler
    {
        public static void WindowLoaded(Window window)
        {
            IntPtr Handle = WindowInterop.Handle(window);

            //ShowInTaskbar = false : causing issue with windows10-windows11 Taskview.
            WindowOperations.RemoveWindowFromTaskbar(Handle);

            //this hides the window from taskbar and also fixes crash when win10-win11 taskview is launched. 
            window.ShowInTaskbar = true;
            window.ShowInTaskbar = false;
        }

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

        public static void EdgePlayerInitializationCompleted(object? sender, CoreWebView2InitializationCompletedEventArgs e)
        {
            Internal.EdgePlayer.CoreWebView2.DOMContentLoaded += EdgePlayerDOMContentLoaded;
        }

        public static void EdgePlayerDOMContentLoaded(object? sender, CoreWebView2DOMContentLoadedEventArgs e)
        {
            Internal.EdgePlayer.CoreWebView2.ExecuteScriptAsync("document.getElementsByTagName('video')[0].requestFullscreen();");
            Internal.EdgePlayer.CoreWebView2.ExecuteScriptAsync("document.getElementsByTagName('video')[0].controls = false;");
            Internal.EdgePlayer.CoreWebView2.ExecuteScriptAsync("document.getElementsByTagName('video')[0].loop = true;");
        }
    }
}