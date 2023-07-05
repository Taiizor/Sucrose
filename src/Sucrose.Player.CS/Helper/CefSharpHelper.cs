using CefSharp;
using SPCSMI = Sucrose.Player.CS.Manage.Internal;
using SSEST = Sucrose.Space.Enum.StretchType;

namespace Sucrose.Player.CS.Helper
{
    internal static class CefSharpHelper
    {
        public static void Pause()
        {
            SPCSMI.CefPlayer.ExecuteScriptAsync("document.getElementsByTagName('video')[0].pause();");
        }

        public static void Play()
        {
            SPCSMI.CefPlayer.ExecuteScriptAsync("document.getElementsByTagName('video')[0].play();");
        }

        public static void Stop()
        {
            SPCSMI.CefPlayer.ExecuteScriptAsync("document.getElementsByTagName('video')[0].stop();"); //Not Working
        }

        public static void SetLoop(bool State)
        {
            SPCSMI.CefPlayer.ExecuteScriptAsync($"document.getElementsByTagName('video')[0].loop = {State.ToString().ToLower()};"); //Not Working
        }

        public static void SetVolume(int Volume)
        {
            SPCSMI.CefPlayer.ExecuteScriptAsync($"document.getElementsByTagName('video')[0].volume = {(Volume / 100d).ToString().Replace(" ", ".").Replace(",", ".")};");
        }

        public static void SetStretch(SSEST Stretch)
        {
            switch (Stretch)
            {
                case SSEST.None:
                    SPCSMI.CefPlayer.ExecuteScriptAsync("document.getElementsByTagName('video')[0].style.objectFit = \"none\";");
                    break;
                case SSEST.Fill:
                    SPCSMI.CefPlayer.ExecuteScriptAsync("document.getElementsByTagName('video')[0].style.objectFit = \"fill\";");
                    break;
                case SSEST.Uniform:
                    SPCSMI.CefPlayer.ExecuteScriptAsync("document.getElementsByTagName('video')[0].style.objectFit = \"contain\";");
                    break;
                case SSEST.UniformToFill:
                    SPCSMI.CefPlayer.ExecuteScriptAsync("document.getElementsByTagName('video')[0].style.objectFit = \"cover\";");
                    break;
                default:
                    break;
            }
        }
    }
}