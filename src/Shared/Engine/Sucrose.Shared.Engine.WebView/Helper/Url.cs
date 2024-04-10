using SSEHS = Sucrose.Shared.Engine.Helper.Source;
using SSEWVES = Sucrose.Shared.Engine.WebView.Extension.Screenshot;
using SSEWVHM = Sucrose.Shared.Engine.WebView.Helper.Management;
using SSEWVMI = Sucrose.Shared.Engine.WebView.Manage.Internal;
using SWEACAM = Skylark.Wing.Extension.AudioController.AudioManager;
using SWEVPCAM = Skylark.Wing.Extension.VideoPlayerController.AudioManager;

namespace Sucrose.Shared.Engine.WebView.Helper
{
    internal static class Url
    {
        public static async void Play()
        {
            if (!SSEWVMI.State)
            {
                SSEWVMI.State = true;

                SSEWVMI.WebEngine.Source = new(SSEWVMI.Url);
            }
        }

        public static async void Pause()
        {
            if (SSEWVMI.State)
            {
                SSEWVMI.State = false;

                SSEWVMI.WebEngine.CoreWebView2.NavigateToString(SSEHS.GetImageContent(await SSEWVES.Capture()));
            }
        }

        public static async void SetVolume(int Volume)
        {
            if (SSEWVMI.Processes.Any())
            {
                foreach (int Process in SSEWVMI.Processes.ToList())
                {
                    try
                    {
                        SWEVPCAM.SetApplicationVolume(Process, Volume);
                    }
                    catch
                    {
                        try
                        {
                            SWEACAM.SetApplicationVolume(Process, Volume);
                        }
                        catch { }
                    }
                }
            }

            if (SSEWVMI.Try < 3)
            {
                await Task.Run(() =>
                {
                    SSEWVMI.Try++;
                    SSEWVHM.SetProcesses();
                });
            }
        }
    }
}