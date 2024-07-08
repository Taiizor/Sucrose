using CefSharp;
using SSDEST = Sucrose.Shared.Dependency.Enum.StretchType;
using SSECSHE = Sucrose.Shared.Engine.CefSharp.Helper.Evaluate;
using SSECSHM = Sucrose.Shared.Engine.CefSharp.Helper.Management;
using SSECSMI = Sucrose.Shared.Engine.CefSharp.Manage.Internal;
using SSEHD = Sucrose.Shared.Engine.Helper.Data;
using SSEHS = Sucrose.Shared.Engine.Helper.Source;
using SSWW = Sucrose.Shared.Watchdog.Watch;

namespace Sucrose.Shared.Engine.CefSharp.Helper
{
    internal static class Video
    {
        public static async void Load()
        {
            try
            {
                //SSECSMI.CefEngine.ExecuteScriptAsync("document.getElementsByTagName('video')[0].webkitRequestFullscreen();");
                //SSECSMI.CefEngine.ExecuteScriptAsync("document.getElementsByTagName('video')[0].requestFullscreen();");
                SSECSMI.CefEngine.ExecuteScriptAsync("document.getElementsByTagName('video')[0].controls = false;");
                SSECSMI.CefEngine.ExecuteScriptAsync("document.getElementsByTagName('video')[0].loop = true;");
                SSECSMI.CefEngine.ExecuteScriptAsync(SSEHS.GetVideoStyle());

                SSECSMI.CefEngine.ExecuteScriptAsync("document.getElementsByTagName('video')[0].style = \"position: fixed; top: 0; left: 0; width: 100%; height: 100%; z-index: 9999;\";");

                SetStretch(SSEHD.GetStretch());
                SetVolume(SSEHD.GetVolume());
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
                SSECSMI.CefEngine.ExecuteScriptAsync("document.getElementsByTagName('video')[0].play();");
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
                SSECSMI.CefEngine.ExecuteScriptAsync("document.getElementsByTagName('video')[0].pause();");
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
                string Duration = await SSECSHE.ScriptString($"document.getElementsByTagName('video')[0].duration");
                string Current = await SSECSHE.ScriptString($"document.getElementsByTagName('video')[0].currentTime");

                return Current.Equals(Duration);
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
                SSECSMI.CefEngine.ExecuteScriptAsync($"document.getElementsByTagName('video')[0].loop = {State.ToString().ToLower()};");

                if (State)
                {
                    bool Ended = await GetEnd();

                    if (Ended)
                    {
                        Play();
                    }
                }
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
                SSECSMI.CefEngine.ExecuteScriptAsync($"document.getElementsByTagName('video')[0].volume = {(Volume / 100d).ToString().Replace(" ", ".").Replace(",", ".")};");

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

        public static async void SetStretch(SSDEST Stretch)
        {
            try
            {
                switch (Stretch)
                {
                    case SSDEST.None:
                        SSECSMI.CefEngine.ExecuteScriptAsync("document.getElementsByTagName('video')[0].style.objectFit = \"none\";");
                        break;
                    case SSDEST.Fill:
                        SSECSMI.CefEngine.ExecuteScriptAsync("document.getElementsByTagName('video')[0].style.objectFit = \"fill\";");
                        break;
                    case SSDEST.Uniform:
                        SSECSMI.CefEngine.ExecuteScriptAsync("document.getElementsByTagName('video')[0].style.objectFit = \"contain\";");
                        break;
                    case SSDEST.UniformToFill:
                        SSECSMI.CefEngine.ExecuteScriptAsync("document.getElementsByTagName('video')[0].style.objectFit = \"cover\";");
                        break;
                    default:
                        break;
                }
            }
            catch (Exception Exception)
            {
                await SSWW.Watch_CatchException(Exception);
            }
        }
    }
}