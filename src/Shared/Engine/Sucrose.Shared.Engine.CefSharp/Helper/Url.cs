using SSECSHM = Sucrose.Shared.Engine.CefSharp.Helper.Management;
using SSECSMI = Sucrose.Shared.Engine.CefSharp.Manage.Internal;
using SWEACAM = Skylark.Wing.Extension.AudioController.AudioManager;
using SWEVPCAM = Skylark.Wing.Extension.VideoPlayerController.AudioManager;
using SWNM = Skylark.Wing.Native.Methods;

namespace Sucrose.Shared.Engine.CefSharp.Helper
{
    internal static class Url
    {
        public static void Play()
        {
            if (!SSECSMI.State)
            {
                SSECSMI.State = true;

                //SSECSMI.CefEngine.Address = SSECSMI.Url;

                if (SSECSMI.Processes.Any())
                {
                    foreach (int Process in SSECSMI.Processes.ToList())
                    {
                        _ = SWNM.DebugActiveProcessStop((uint)Process);
                    }
                }

                //if (SSEMI.IntermediateD3DWindow > 0)
                //{
                //    _ = SWNM.DebugActiveProcessStop((uint)SSEMI.IntermediateD3DWindow);
                //}
            }
        }

        public static void Pause()
        {
            if (SSECSMI.State)
            {
                SSECSMI.State = false;

                //string Path = SSEHS.GetImageContentPath();

                //SSEHS.WriteImageContent(Path, SSECSES.Capture());

                //SSECSMI.CefEngine.Address = SSEHS.GetSource(Path).ToString();

                if (SSECSMI.Processes.Any())
                {
                    foreach (int Process in SSECSMI.Processes.ToList())
                    {
                        _ = SWNM.DebugActiveProcess((uint)Process);
                    }
                }

                //if (SSEMI.IntermediateD3DWindow > 0)
                //{
                //    _ = SWNM.DebugActiveProcess((uint)SSEMI.IntermediateD3DWindow);
                //}
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