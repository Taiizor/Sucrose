using CefSharp;
using SDEST = Sucrose.Dependency.Enum.StretchType;
using SECSMI = Sucrose.Engine.CS.Manage.Internal;

namespace Sucrose.Engine.CS.Helper
{
    internal static class Video
    {
        public static void Pause()
        {
            SECSMI.CefEngine.ExecuteScriptAsync("document.getElementsByTagName('video')[0].pause();");
        }

        public static void Play()
        {
            SECSMI.CefEngine.ExecuteScriptAsync("document.getElementsByTagName('video')[0].play();");
        }

        public static async Task<bool> GetEnd()
        {
            JavascriptResponse Response;
            string Current = string.Empty;
            string Duration = string.Empty;

            if (SECSMI.CefEngine.CanExecuteJavascriptInMainFrame)
            {
                Response = await SECSMI.CefEngine.EvaluateScriptAsync($"document.getElementsByTagName('video')[0].duration");

                if (Response.Success)
                {
                    Duration = Response.Result.ToString();
                }

                Response = await SECSMI.CefEngine.EvaluateScriptAsync($"document.getElementsByTagName('video')[0].currentTime");

                if (Response.Success)
                {
                    Current = Response.Result.ToString();
                }
            }
            else
            {
                IFrame Frame = SECSMI.CefEngine.GetMainFrame();

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
            SECSMI.CefEngine.ExecuteScriptAsync($"document.getElementsByTagName('video')[0].loop = {State.ToString().ToLower()};");

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
            SECSMI.CefEngine.ExecuteScriptAsync($"document.getElementsByTagName('video')[0].volume = {(Volume / 100d).ToString().Replace(" ", ".").Replace(",", ".")};");
        }

        public static void SetStretch(SDEST Stretch)
        {
            switch (Stretch)
            {
                case SDEST.None:
                    SECSMI.CefEngine.ExecuteScriptAsync("document.getElementsByTagName('video')[0].style.objectFit = \"none\";");
                    break;
                case SDEST.Fill:
                    SECSMI.CefEngine.ExecuteScriptAsync("document.getElementsByTagName('video')[0].style.objectFit = \"fill\";");
                    break;
                case SDEST.Uniform:
                    SECSMI.CefEngine.ExecuteScriptAsync("document.getElementsByTagName('video')[0].style.objectFit = \"contain\";");
                    break;
                case SDEST.UniformToFill:
                    SECSMI.CefEngine.ExecuteScriptAsync("document.getElementsByTagName('video')[0].style.objectFit = \"cover\";");
                    break;
                default:
                    break;
            }
        }
    }
}