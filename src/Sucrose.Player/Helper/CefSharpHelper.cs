using CefSharp;
using Sucrose.Player.Manage;

namespace Sucrose.Player.Helper
{
    internal static class CefSharpHelper
    {
        public static void Pause()
        {
            Internal.CefPlayer.ExecuteScriptAsync("document.getElementsByTagName('video')[0].pause();");
        }

        public static void Play()
        {
            Internal.CefPlayer.ExecuteScriptAsync("document.getElementsByTagName('video')[0].play();");
        }

        public static void Stop()
        {
            Internal.CefPlayer.ExecuteScriptAsync("document.getElementsByTagName('video')[0].stop();"); //Not Working
        }

        public static void SetVolume(int Volume)
        {
            Internal.CefPlayer.ExecuteScriptAsync($"document.getElementsByTagName('video')[0].volume = {(double)Volume / 100};");
        }
    }
}