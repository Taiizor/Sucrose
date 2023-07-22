using CefSharp;
using SECSHE = Sucrose.Engine.CS.Helper.Evaluate;
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

        public static void First()
        {
            SECSMI.CefEngine.ExecuteScriptAsync("playFirst();");
        }

        public static async Task<bool> GetEnd()
        {
            string State = await SECSHE.ScriptString($"checkVideoEnded();");

            bool.TryParse(State, out bool Result);

            return Result;
        }

        public static async Task<bool> GetPlay()
        {
            string State = await SECSHE.ScriptString($"checkPlayingStatus();");

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