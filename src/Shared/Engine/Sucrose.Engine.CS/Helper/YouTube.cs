using CefSharp;
using SECSMI = Sucrose.Engine.CS.Manage.Internal;

namespace Sucrose.Engine.CS.Helper
{
    internal static class YouTube
    {
        public static void Pause()
        {
            SECSMI.CefEngine.ExecuteScriptAsync("pauseVideo();");
        }

        public static void Play()
        {
            SECSMI.CefEngine.ExecuteScriptAsync("playVideo();");
        }

        public static async void Play2()
        {
            bool Playing = await GetPlay();

            if (!Playing)
            {
                Play();
            }
        }

        public static async Task<bool> GetEnd()
        {
            JavascriptResponse Response;
            string State = string.Empty;

            if (SECSMI.CefEngine.CanExecuteJavascriptInMainFrame)
            {
                Response = await SECSMI.CefEngine.EvaluateScriptAsync($"checkVideoEnded();");

                if (Response.Success)
                {
                    State = Response.Result.ToString();
                }
            }
            else
            {
                IFrame Frame = SECSMI.CefEngine.GetMainFrame();

                Response = await Frame.EvaluateScriptAsync($"checkVideoEnded();");

                if (Response.Success)
                {
                    State = Response.Result.ToString();
                }
            }

            bool.TryParse(State, out bool Result);

            return Result;
        }

        public static async Task<bool> GetPlay()
        {
            JavascriptResponse Response;
            string State = string.Empty;

            if (SECSMI.CefEngine.CanExecuteJavascriptInMainFrame)
            {
                Response = await SECSMI.CefEngine.EvaluateScriptAsync($"checkPlayingStatus();");

                if (Response.Success)
                {
                    State = Response.Result.ToString();
                }
            }
            else
            {
                IFrame Frame = SECSMI.CefEngine.GetMainFrame();

                Response = await Frame.EvaluateScriptAsync($"checkPlayingStatus();");

                if (Response.Success)
                {
                    State = Response.Result.ToString();
                }
            }

            bool.TryParse(State, out bool Result);

            return Result;
        }

        public static void SetLoop(bool State)
        {
            SECSMI.CefEngine.ExecuteScriptAsync($"setLoop({State.ToString().ToLower()});");
        }

        public static void SetVolume(int Volume)
        {
            SECSMI.CefEngine.ExecuteScriptAsync($"setVolume({Volume});");
        }

        public static void SetShuffle(bool State)
        {
            SECSMI.CefEngine.ExecuteScriptAsync($"setShuffle({State.ToString().ToLower()});");
        }
    }
}