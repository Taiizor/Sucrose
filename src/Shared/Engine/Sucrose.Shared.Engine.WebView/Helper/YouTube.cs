using SSEWVMI = Sucrose.Shared.Engine.WebView.Manage.Internal;

namespace Sucrose.Shared.Engine.WebView.Helper
{
    internal static class YouTube
    {
        public static async void Pause()
        {
            await SSEWVMI.WebEngine.CoreWebView2.ExecuteScriptAsync("pauseVideo();");
        }

        public static async void Play()
        {
            await SSEWVMI.WebEngine.CoreWebView2.ExecuteScriptAsync("playVideo();");
        }

        public static async void Play2()
        {
            bool Playing = await GetPlay();

            if (!Playing)
            {
                Play();
            }
        }

        public static async void First()
        {
            await SSEWVMI.WebEngine.CoreWebView2.ExecuteScriptAsync("playFirst();");
        }

        public static async Task<bool> GetEnd()
        {
            string State = await SSEWVMI.WebEngine.CoreWebView2.ExecuteScriptAsync($"checkVideoEnded();");

            bool.TryParse(State, out bool Result);

            return Result;
        }

        public static async Task<bool> GetPlay()
        {
            string State = await SSEWVMI.WebEngine.CoreWebView2.ExecuteScriptAsync($"checkPlayingStatus();");

            bool.TryParse(State, out bool Result);

            return Result;
        }

        public static async void SetLoop(bool State)
        {
            await SSEWVMI.WebEngine.CoreWebView2.ExecuteScriptAsync($"setLoop({State.ToString().ToLower()});");
        }

        public static async void SetVolume(int Volume)
        {
            await SSEWVMI.WebEngine.CoreWebView2.ExecuteScriptAsync($"setVolume({Volume});");
        }

        public static async void SetShuffle(bool State)
        {
            await SSEWVMI.WebEngine.CoreWebView2.ExecuteScriptAsync($"setShuffle({State.ToString().ToLower()});");
        }
    }
}