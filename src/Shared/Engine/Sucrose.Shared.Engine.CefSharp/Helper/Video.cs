using CefSharp;
using SSDEST = Sucrose.Shared.Dependency.Enum.StretchType;
using SSECSHE = Sucrose.Shared.Engine.CefSharp.Helper.Evaluate;
using SSECSHM = Sucrose.Shared.Engine.CefSharp.Helper.Management;
using SSECSMI = Sucrose.Shared.Engine.CefSharp.Manage.Internal;

namespace Sucrose.Shared.Engine.CefSharp.Helper
{
    internal static class Video
    {
        public static void Pause()
        {
            SSECSMI.CefEngine.ExecuteScriptAsync("document.getElementsByTagName('video')[0].pause();");
        }

        public static void Play()
        {
            SSECSMI.CefEngine.ExecuteScriptAsync("document.getElementsByTagName('video')[0].play();");
        }

        public static async Task<bool> GetEnd()
        {
            string Duration = await SSECSHE.ScriptString($"document.getElementsByTagName('video')[0].duration");
            string Current = await SSECSHE.ScriptString($"document.getElementsByTagName('video')[0].currentTime");

            return Current.Equals(Duration);
        }

        public static async void SetLoop(bool State)
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

        public static void SetStretch(SSDEST Stretch)
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

        public static async void SetVolume(int Volume)
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
    }
}