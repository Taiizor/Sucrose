using SSDEST = Sucrose.Shared.Dependency.Enum.StretchType;
using SSEHD = Sucrose.Shared.Engine.Helper.Data;
using SSEWVMI = Sucrose.Shared.Engine.WebView.Manage.Internal;
using SSWEW = Sucrose.Shared.Watchdog.Extension.Watch;

namespace Sucrose.Shared.Engine.WebView.Helper
{
    internal static class Gif
    {
        public static async void Load()
        {
            try
            {
                SetStretch(SSEHD.GetStretch());
                SetLoop(SSEHD.GetLoop());
            }
            catch (Exception Exception)
            {
                await SSWEW.Watch_CatchException(Exception);
            }
        }

        public static async void Play()
        {
            try
            {
                await SSEWVMI.WebEngine.ExecuteScriptAsync("SucrosePlay();");
            }
            catch (Exception Exception)
            {
                await SSWEW.Watch_CatchException(Exception);
            }
        }

        public static async void Pause()
        {
            try
            {
                await SSEWVMI.WebEngine.ExecuteScriptAsync("SucrosePause();");
            }
            catch (Exception Exception)
            {
                await SSWEW.Watch_CatchException(Exception);
            }
        }

        public static async void PlayPause()
        {
            try
            {
                await SSEWVMI.WebEngine.ExecuteScriptAsync("SucrosePlayPause();");
            }
            catch (Exception Exception)
            {
                await SSWEW.Watch_CatchException(Exception);
            }
        }

        public static async void SetLoop(bool State)
        {
            try
            {
                await SSEWVMI.WebEngine.ExecuteScriptAsync($"SucroseLoopMode({State.ToString().ToLower()});");
            }
            catch (Exception Exception)
            {
                await SSWEW.Watch_CatchException(Exception);
            }
        }

        public static async void SetStretch(SSDEST Stretch)
        {
            try
            {
                await SSEWVMI.WebEngine.ExecuteScriptAsync($"SucroseStretchMode('{Stretch}');");
            }
            catch (Exception Exception)
            {
                await SSWEW.Watch_CatchException(Exception);
            }
        }
    }
}