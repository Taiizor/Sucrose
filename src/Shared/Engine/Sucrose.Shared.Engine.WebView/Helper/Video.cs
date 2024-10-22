using SSDEST = Sucrose.Shared.Dependency.Enum.StretchType;
using SSEHD = Sucrose.Shared.Engine.Helper.Data;
using SSEHS = Sucrose.Shared.Engine.Helper.Source;
using SSEWVHM = Sucrose.Shared.Engine.WebView.Helper.Management;
using SSEWVMI = Sucrose.Shared.Engine.WebView.Manage.Internal;
using SSWEW = Sucrose.Shared.Watchdog.Extension.Watch;

namespace Sucrose.Shared.Engine.WebView.Helper
{
    internal static class Video
    {
        public static async void Load()
        {
            try
            {
                await SSEWVMI.WebEngine.CoreWebView2.ExecuteScriptAsync("document.getElementsByTagName('video')[0].requestFullscreen();");
                await SSEWVMI.WebEngine.CoreWebView2.ExecuteScriptAsync("document.getElementsByTagName('video')[0].controls = false;");
                await SSEWVMI.WebEngine.CoreWebView2.ExecuteScriptAsync("document.getElementsByTagName('video')[0].loop = true;");
                await SSEWVMI.WebEngine.CoreWebView2.ExecuteScriptAsync(SSEHS.GetVideoStyle());

                SetStretch(SSEHD.GetStretch());
                SetVolume(SSEHD.GetVolume());
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
                await SSEWVMI.WebEngine.CoreWebView2.ExecuteScriptAsync("document.getElementsByTagName('video')[0].play();");
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
                await SSEWVMI.WebEngine.CoreWebView2.ExecuteScriptAsync("document.getElementsByTagName('video')[0].pause();");
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
                string Duration = await SSEWVMI.WebEngine.CoreWebView2.ExecuteScriptAsync($"document.getElementsByTagName('video')[0].duration");
                string Current = await SSEWVMI.WebEngine.CoreWebView2.ExecuteScriptAsync($"document.getElementsByTagName('video')[0].currentTime");

                return Current.Equals(Duration);
            }
            catch (Exception Exception)
            {
                await SSWEW.Watch_CatchException(Exception);

                return false;
            }
        }

        public static async Task<string> GetVideo()
        {
            try
            {
                return await SSEWVMI.WebEngine.CoreWebView2.ExecuteScriptAsync($"document.getElementsByTagName('video')[0].src");
            }
            catch (Exception Exception)
            {
                await SSWEW.Watch_CatchException(Exception);

                return string.Empty;
            }
        }

        public static async void SetLoop(bool State)
        {
            try
            {
                await SSEWVMI.WebEngine.CoreWebView2.ExecuteScriptAsync($"document.getElementsByTagName('video')[0].loop = {State.ToString().ToLower()};");

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
                await SSWEW.Watch_CatchException(Exception);
            }
        }

        public static async void SetVolume(int Volume)
        {
            try
            {
                await SSEWVMI.WebEngine.CoreWebView2.ExecuteScriptAsync($"document.getElementsByTagName('video')[0].volume = {(Volume / 100d).ToString().Replace(" ", ".").Replace(",", ".")};");

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

        public static async void SetVideo(string Video)
        {
            try
            {
                await SSEWVMI.WebEngine.CoreWebView2.ExecuteScriptAsync($"document.getElementsByTagName('video')[0].src = '{Video}';");
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
                switch (Stretch)
                {
                    case SSDEST.None:
                        await SSEWVMI.WebEngine.ExecuteScriptAsync("document.getElementsByTagName('video')[0].style.objectFit = \"none\";");
                        break;
                    case SSDEST.Fill:
                        await SSEWVMI.WebEngine.ExecuteScriptAsync("document.getElementsByTagName('video')[0].style.objectFit = \"fill\";");
                        break;
                    case SSDEST.Uniform:
                        await SSEWVMI.WebEngine.ExecuteScriptAsync("document.getElementsByTagName('video')[0].style.objectFit = \"contain\";");
                        break;
                    case SSDEST.UniformToFill:
                        await SSEWVMI.WebEngine.ExecuteScriptAsync("document.getElementsByTagName('video')[0].style.objectFit = \"cover\";");
                        break;
                    default:
                        break;
                }
            }
            catch (Exception Exception)
            {
                await SSWEW.Watch_CatchException(Exception);
            }
        }
    }
}