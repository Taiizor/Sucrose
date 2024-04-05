using SSDEST = Sucrose.Shared.Dependency.Enum.StretchType;
using SSEWVMI = Sucrose.Shared.Engine.WebView.Manage.Internal;

namespace Sucrose.Shared.Engine.WebView.Helper
{
    internal static class Gif
    {
        public static async void Play()
        {
            await SSEWVMI.WebEngine.ExecuteScriptAsync("SucrosePlay();");
        }

        public static async void Pause()
        {
            await SSEWVMI.WebEngine.ExecuteScriptAsync("SucrosePause();");
        }

        public static async void PlayPause()
        {
            await SSEWVMI.WebEngine.ExecuteScriptAsync("SucrosePlayPause();");
        }

        public static async void SetLoop(bool State)
        {
            await SSEWVMI.WebEngine.ExecuteScriptAsync($"SucroseLoopMode({State.ToString().ToLower()});");
        }

        public static async void SetStretch(SSDEST Stretch)
        {
            await SSEWVMI.WebEngine.ExecuteScriptAsync($"SucroseStretchMode('{Stretch}');");
        }
    }
}