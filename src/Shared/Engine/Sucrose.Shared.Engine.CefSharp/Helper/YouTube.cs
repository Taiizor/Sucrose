using CefSharp;
using SSECSHE = Sucrose.Shared.Engine.CefSharp.Helper.Evaluate;
using SSECSHM = Sucrose.Shared.Engine.CefSharp.Helper.Management;
using SSECSMI = Sucrose.Shared.Engine.CefSharp.Manage.Internal;
using SSEHD = Sucrose.Shared.Engine.Helper.Data;
using SSWW = Sucrose.Shared.Watchdog.Watch;

namespace Sucrose.Shared.Engine.CefSharp.Helper
{
    internal static class YouTube
    {
        public static async void Load()
        {
            try
            {
                SSECSMI.CefEngine.ExecuteScriptAsync($"setVolume({SSEHD.GetVolume()});");
                SSECSMI.CefEngine.ExecuteScriptAsync("toggleFullScreen();");
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
                SSECSMI.CefEngine.ExecuteScriptAsync("playVideo();");
            }
            catch (Exception Exception)
            {
                await SSWW.Watch_CatchException(Exception);
            }
        }

        public static async void First()
        {
            try
            {
                SSECSMI.CefEngine.ExecuteScriptAsync("playFirst();");
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
                SSECSMI.CefEngine.ExecuteScriptAsync("pauseVideo();");
            }
            catch (Exception Exception)
            {
                await SSWW.Watch_CatchException(Exception);
            }
        }

        public static async void Play2()
        {
            try
            {
                bool Playing = await GetPlay();

                if (!Playing)
                {
                    Play();
                }
            }
            catch (Exception Exception)
            {
                await SSWW.Watch_CatchException(Exception);
            }
        }

        public static async Task<bool> GetEnd()
        {
            try
            {
                string State = await SSECSHE.ScriptString($"checkVideoEnded();");

                bool.TryParse(State, out bool Result);

                return Result;
            }
            catch (Exception Exception)
            {
                await SSWW.Watch_CatchException(Exception);

                return false;
            }
        }

        public static async Task<bool> GetPlay()
        {
            try
            {
                string State = await SSECSHE.ScriptString($"checkPlayingStatus();");

                bool.TryParse(State, out bool Result);

                return Result;
            }
            catch (Exception Exception)
            {
                await SSWW.Watch_CatchException(Exception);

                return false;
            }
        }

        public static async void SetLoop(bool State)
        {
            try
            {
                SSECSMI.CefEngine.ExecuteScriptAsync($"setLoop({State.ToString().ToLower()});");
            }
            catch (Exception Exception)
            {
                await SSWW.Watch_CatchException(Exception);
            }
        }

        public static async void SetVolume(int Volume)
        {
            try
            {
                SSECSMI.CefEngine.ExecuteScriptAsync($"setVolume({Volume});");

                if (SSECSMI.Try < 3)
                {
                    await Task.Run(() =>
                    {
                        SSECSMI.Try++;
                        SSECSHM.SetProcesses();
                    });
                }
            }
            catch (Exception Exception)
            {
                await SSWW.Watch_CatchException(Exception);
            }
        }

        public static async void SetShuffle(bool State)
        {
            try
            {
                SSECSMI.CefEngine.ExecuteScriptAsync($"setShuffle({State.ToString().ToLower()});");
            }
            catch (Exception Exception)
            {
                await SSWW.Watch_CatchException(Exception);
            }
        }
    }
}