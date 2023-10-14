using SSDEST = Sucrose.Shared.Dependency.Enum.StretchType;
using SSEWVMI = Sucrose.Shared.Engine.WebView.Manage.Internal;

namespace Sucrose.Shared.Engine.WebView.Helper
{
    internal static class Gif
    {
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