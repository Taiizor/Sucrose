using CefSharp;
using SSECSHM = Sucrose.Shared.Engine.CefSharp.Helper.Management;
using SSECSMI = Sucrose.Shared.Engine.CefSharp.Manage.Internal;
using SSECSES = Sucrose.Shared.Engine.CefSharp.Extension.Screenshot;
using SSEHS = Sucrose.Shared.Engine.Helper.Source;
using SWEACAM = Skylark.Wing.Extension.AudioController.AudioManager;
using SWEVPCAM = Skylark.Wing.Extension.VideoPlayerController.AudioManager;

namespace Sucrose.Shared.Engine.CefSharp.Helper
{
    internal static class Url
    {
        public static async void Play()
        {
            if (!SSECSMI.State)
            {
                SSECSMI.State = true;

                SSECSMI.CefEngine.Address = SSECSMI.Url;
            }
        }

        public static async void Pause()
        {
            if (SSECSMI.State)
            {
                SSECSMI.State = false;

                SSECSMI.CefEngine.LoadHtml(SSEHS.GetImageContent(await SSECSES.Capture()));
            }
        }

        public static async void SetVolume(int Volume)
        {
            if (SSECSMI.Processes.Any())
            {
                foreach (int Process in SSECSMI.Processes.ToList())
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