using CefSharp;
using SECSHE = Sucrose.Engine.CS.Helper.Evaluate;
using SECSMI = Sucrose.Engine.CS.Manage.Internal;
using SSDEST = Sucrose.Shared.Dependency.Enum.StretchType;

namespace Sucrose.Engine.CS.Helper
{
    internal static class Video
    {
        public static void Pause()
        {
            SECSMI.CefEngine.ExecuteScriptAsync("document.getElementsByTagName('video')[0].pause();");
        }

        public static void Play()
        {
            SECSMI.CefEngine.ExecuteScriptAsync("document.getElementsByTagName('video')[0].play();");
        }

        public static async Task<bool> GetEnd()
        {
            string Duration = await SECSHE.ScriptString($"document.getElementsByTagName('video')[0].duration");
            string Current = await SECSHE.ScriptString($"document.getElementsByTagName('video')[0].currentTime");

            return Current.Equals(Duration);
        }

        public static async void SetLoop(bool State)
        {
            SECSMI.CefEngine.ExecuteScriptAsync($"document.getElementsByTagName('video')[0].loop = {State.ToString().ToLower()};");

            if (State)
            {
                bool Ended = await GetEnd();

                if (Ended)
                {
                    Play();
                }
            }
        }

        public static void SetVolume(int Volume)
        {
            SECSMI.CefEngine.ExecuteScriptAsync($"document.getElementsByTagName('video')[0].volume = {(Volume / 100d).ToString().Replace(" ", ".").Replace(",", ".")};");
        }

        public static void SetStretch(SSDEST Stretch)
        {
            switch (Stretch)
            {
                case SSDEST.None:
                    SECSMI.CefEngine.ExecuteScriptAsync("document.getElementsByTagName('video')[0].style.objectFit = \"none\";");
                    break;
                case SSDEST.Fill:
                    SECSMI.CefEngine.ExecuteScriptAsync("document.getElementsByTagName('video')[0].style.objectFit = \"fill\";");
                    break;
                case SSDEST.Uniform:
                    SECSMI.CefEngine.ExecuteScriptAsync("document.getElementsByTagName('video')[0].style.objectFit = \"contain\";");
                    break;
                case SSDEST.UniformToFill:
                    SECSMI.CefEngine.ExecuteScriptAsync("document.getElementsByTagName('video')[0].style.objectFit = \"cover\";");
                    break;
                default:
                    break;
            }
        }
    }
}