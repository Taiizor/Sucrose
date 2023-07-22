using CefSharp;
using SSECSHE = Sucrose.Shared.Engine.CefSharp.Helper.Evaluate;
using SSECSMI = Sucrose.Shared.Engine.CefSharp.Manage.Internal;

namespace Sucrose.Shared.Engine.CefSharp.Helper
{
    internal static class YouTube
    {
        public static void Pause()
        {
            SSECSMI.CefEngine.ExecuteScriptAsync("pauseVideo();");
        }

        public static void Play()
        {
            SSECSMI.CefEngine.ExecuteScriptAsync("playVideo();");
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
            SSECSMI.CefEngine.ExecuteScriptAsync("playFirst();");
        }

        public static async Task<bool> GetEnd()
        {
            string State = await SSECSHE.ScriptString($"checkVideoEnded();");

            bool.TryParse(State, out bool Result);

            return Result;
        }

        public static async Task<bool> GetPlay()
        {
            string State = await SSECSHE.ScriptString($"checkPlayingStatus();");

            bool.TryParse(State, out bool Result);

            return Result;
        }

        public static void SetLoop(bool State)
        {
            SSECSMI.CefEngine.ExecuteScriptAsync($"setLoop({State.ToString().ToLower()});");
        }

        public static void SetVolume(int Volume)
        {
            SSECSMI.CefEngine.ExecuteScriptAsync($"setVolume({Volume});");
        }

        public static void SetShuffle(bool State)
        {
            SSECSMI.CefEngine.ExecuteScriptAsync($"setShuffle({State.ToString().ToLower()});");
        }
    }
}