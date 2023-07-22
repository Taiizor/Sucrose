using SSDEST = Sucrose.Shared.Dependency.Enum.StretchType;
using SSEWVMI = Sucrose.Shared.Engine.WebView.Manage.Internal;

namespace Sucrose.Shared.Engine.WebView.Helper
{
    internal static class Video
    {
        public static async void Pause()
        {
            await SSEWVMI.WebEngine.CoreWebView2.ExecuteScriptAsync("document.getElementsByTagName('video')[0].pause();");
        }

        public static async void Play()
        {
            await SSEWVMI.WebEngine.CoreWebView2.ExecuteScriptAsync("document.getElementsByTagName('video')[0].play();");
        }

        public static async Task<string> GetVideo()
        {
            return await SSEWVMI.WebEngine.CoreWebView2.ExecuteScriptAsync($"document.getElementsByTagName('video')[0].src");
        }

        public static async Task<bool> GetEnd()
        {
            string Duration = await SSEWVMI.WebEngine.CoreWebView2.ExecuteScriptAsync($"document.getElementsByTagName('video')[0].duration");
            string Current = await SSEWVMI.WebEngine.CoreWebView2.ExecuteScriptAsync($"document.getElementsByTagName('video')[0].currentTime");

            return Current.Equals(Duration);
        }

        public static void SetVideo(string Video)
        {
            SSEWVMI.WebEngine.CoreWebView2.ExecuteScriptAsync($"document.getElementsByTagName('video')[0].src = '{Video}';");
        }

        public static async void SetLoop(bool State)
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

        public static async void SetVolume(int Volume)
        {
            await SSEWVMI.WebEngine.CoreWebView2.ExecuteScriptAsync($"document.getElementsByTagName('video')[0].volume = {(Volume / 100d).ToString().Replace(" ", ".").Replace(",", ".")};");
        }

        public static async void SetStretch(SSDEST Stretch)
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
    }
}