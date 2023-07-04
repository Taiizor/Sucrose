using Sucrose.Player.WV.Manage;

namespace Sucrose.Player.WV.Helper
{
    internal static class WebView2Helper
    {
        public static void Pause()
        {
            Internal.EdgePlayer.CoreWebView2.ExecuteScriptAsync("document.getElementsByTagName('video')[0].pause();");
        }

        public static void Play()
        {
            Internal.EdgePlayer.CoreWebView2.ExecuteScriptAsync("document.getElementsByTagName('video')[0].play();");
        }

        public static void Stop()
        {
            Internal.EdgePlayer.CoreWebView2.ExecuteScriptAsync("document.getElementsByTagName('video')[0].stop();"); //Not Working
        }

        public static void SetVolume(int Volume)
        {
            Internal.EdgePlayer.CoreWebView2.ExecuteScriptAsync($"document.getElementsByTagName('video')[0].volume = {(double)Volume / 100};");
        }
    }
}