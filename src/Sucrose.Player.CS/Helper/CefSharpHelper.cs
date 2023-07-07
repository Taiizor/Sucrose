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

        public static async Task<bool> GetEnd()
        {
            JavascriptResponse Response;
            string Current = string.Empty;
            string Duration = string.Empty;

            if (SPCSMI.CefPlayer.CanExecuteJavascriptInMainFrame)
            {
                Response = await SPCSMI.CefPlayer.EvaluateScriptAsync($"document.getElementsByTagName('video')[0].duration");

                if (Response.Success)
                {
                    Duration = Response.Result.ToString();
                }

                Response = await SPCSMI.CefPlayer.EvaluateScriptAsync($"document.getElementsByTagName('video')[0].currentTime");

                if (Response.Success)
                {
                    Current = Response.Result.ToString();
                }
            }
            else
            {
                IFrame Frame = SPCSMI.CefPlayer.GetMainFrame();

                Response = await Frame.EvaluateScriptAsync($"document.getElementsByTagName('video')[0].duration");

                if (Response.Success)
                {
                    Duration = Response.Result.ToString();
                }

                Response = await Frame.EvaluateScriptAsync($"document.getElementsByTagName('video')[0].currentTime");

                if (Response.Success)
                {
                    Current = Response.Result.ToString();
                }
            }

            return Current.Equals(Duration);
        }

        public static async void SetLoop(bool State)
        {
            SPCSMI.CefPlayer.ExecuteScriptAsync($"document.getElementsByTagName('video')[0].loop = {State.ToString().ToLower()};");

            if (State)
            {
                bool Ended = await GetEnd();

                if (Ended)
                {
                    Play();
                }
            }
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