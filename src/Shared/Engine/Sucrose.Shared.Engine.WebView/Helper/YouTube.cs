using SSEHD = Sucrose.Shared.Engine.Helper.Data;
using SSEWVHM = Sucrose.Shared.Engine.WebView.Helper.Management;
using SSEWVMI = Sucrose.Shared.Engine.WebView.Manage.Internal;
using SSWEW = Sucrose.Shared.Watchdog.Extension.Watch;

namespace Sucrose.Shared.Engine.WebView.Helper
{
    internal static class YouTube
    {
        public static async void Load()
        {
            try
            {
                await SSEWVMI.WebEngine.CoreWebView2.ExecuteScriptAsync($"setVolume({SSEHD.GetVolume()});");
                await SSEWVMI.WebEngine.CoreWebView2.ExecuteScriptAsync("toggleFullScreen();");
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
                await SSEWVMI.WebEngine.CoreWebView2.ExecuteScriptAsync("playVideo();");
            }
            catch (Exception Exception)
            {
                await SSWEW.Watch_CatchException(Exception);
            }
        }

        public static async void First()
        {
            try
            {
                await SSEWVMI.WebEngine.CoreWebView2.ExecuteScriptAsync("playFirst();");
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
                await SSEWVMI.WebEngine.CoreWebView2.ExecuteScriptAsync("pauseVideo();");
            }
            catch (Exception Exception)
            {
                await SSWEW.Watch_CatchException(Exception);
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
                await SSWEW.Watch_CatchException(Exception);
            }
        }

        public static async Task<bool> GetEnd()
        {
            try
            {
                string State = await SSEWVMI.WebEngine.CoreWebView2.ExecuteScriptAsync($"checkVideoEnded();");

                bool.TryParse(State, out bool Result);

                return Result;
            }
            catch (Exception Exception)
            {
                await SSWEW.Watch_CatchException(Exception);

                return false;
            }
        }

        public static async Task<bool> GetPlay()
        {
            try
            {
                string State = await SSEWVMI.WebEngine.CoreWebView2.ExecuteScriptAsync($"checkPlayingStatus();");

                bool.TryParse(State, out bool Result);

                return Result;
            }
            catch (Exception Exception)
            {
                await SSWEW.Watch_CatchException(Exception);

                return false;
            }
        }

        public static async void SetLoop(bool State)
        {
            try
            {
                await SSEWVMI.WebEngine.CoreWebView2.ExecuteScriptAsync($"setLoop({State.ToString().ToLower()});");
            }
            catch (Exception Exception)
            {
                await SSWEW.Watch_CatchException(Exception);
            }
        }

        public static async void SetVolume(int Volume)
        {
            try
            {
                await SSEWVMI.WebEngine.CoreWebView2.ExecuteScriptAsync($"setVolume({Volume});");

                if (SSEWVMI.Try < 3)
                {
                    await Task.Run(() =>
                    {
                        SSEWVMI.Try++;
                        SSEWVHM.SetProcesses();
                    });
                }
            }
            catch (Exception Exception)
            {
                await SSWEW.Watch_CatchException(Exception);
            }
        }

        public static async void SetShuffle(bool State)
        {
            try
            {
                await SSEWVMI.WebEngine.CoreWebView2.ExecuteScriptAsync($"setShuffle({State.ToString().ToLower()});");
            }
            catch (Exception Exception)
            {
                await SSWEW.Watch_CatchException(Exception);
            }
        }
    }
}