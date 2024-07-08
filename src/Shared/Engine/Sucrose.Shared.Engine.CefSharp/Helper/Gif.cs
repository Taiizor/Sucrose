using CefSharp;
using SSDEST = Sucrose.Shared.Dependency.Enum.StretchType;
using SSECSMI = Sucrose.Shared.Engine.CefSharp.Manage.Internal;
using SSEHD = Sucrose.Shared.Engine.Helper.Data;
using SSWW = Sucrose.Shared.Watchdog.Watch;

namespace Sucrose.Shared.Engine.CefSharp.Helper
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
                await SSWW.Watch_CatchException(Exception);
            }
        }

        public static async void Play()
        {
            try
            {
                SSECSMI.CefEngine.ExecuteScriptAsync("SucrosePlay();");
            }
            catch (Exception Exception)
            {
                await SSWW.Watch_CatchException(Exception);
            }
        }

        public static async void Pause()
        {
            try
            {
                SSECSMI.CefEngine.ExecuteScriptAsync("SucrosePause();");
            }
            catch (Exception Exception)
            {
                await SSWW.Watch_CatchException(Exception);
            }
        }

        public static async void PlayPause()
        {
            try
            {
                SSECSMI.CefEngine.ExecuteScriptAsync("SucrosePlayPause();");
            }
            catch (Exception Exception)
            {
                await SSWW.Watch_CatchException(Exception);
            }
        }

        public static async void SetLoop(bool State)
        {
            try
            {
                SSECSMI.CefEngine.ExecuteScriptAsync($"SucroseLoopMode({State.ToString().ToLower()});");
            }
            catch (Exception Exception)
            {
                await SSWW.Watch_CatchException(Exception);
            }
        }

        public static async void SetStretch(SSDEST Stretch)
        {
            try
            {
                SSECSMI.CefEngine.ExecuteScriptAsync($"SucroseStretchMode('{Stretch}');");
            }
            catch (Exception Exception)
            {
                await SSWW.Watch_CatchException(Exception);
            }
        }
    }
}