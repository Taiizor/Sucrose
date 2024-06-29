using SSEMI = Sucrose.Shared.Engine.Manage.Internal;
using SSEWVHM = Sucrose.Shared.Engine.WebView.Helper.Management;
using SSEWVMI = Sucrose.Shared.Engine.WebView.Manage.Internal;
using SWEACAM = Skylark.Wing.Extension.AudioController.AudioManager;
using SWEVPCAM = Skylark.Wing.Extension.VideoPlayerController.AudioManager;
using SWNM = Skylark.Wing.Native.Methods;

namespace Sucrose.Shared.Engine.WebView.Helper
{
    internal static class Url
    {
        public static void Play()
        {
            if (!SSEWVMI.State)
            {
                SSEWVMI.State = true;

                //SSEWVMI.WebEngine.Source = new(SSEWVMI.Url);

                if (SSEMI.IntermediateD3DWindow > 0)
                {
                    _ = SWNM.DebugActiveProcessStop((uint)SSEMI.IntermediateD3DWindow);
                }
            }
        }

        public static void Pause()
        {
            if (SSEWVMI.State)
            {
                SSEWVMI.State = false;

                //string Path = SSEHS.GetImageContentPath();

                //SSEHS.WriteImageContent(Path, await SSEWVES.Capture());

                //SSEWVMI.WebEngine.Source = SSEHS.GetSource(Path);

                if (SSEMI.IntermediateD3DWindow > 0)
                {
                    _ = SWNM.DebugActiveProcess((uint)SSEMI.IntermediateD3DWindow);
                }
            }
        }

        public static async void SetVolume(int Volume)
        {
            if (SSEMI.Processes.Any())
            {
                foreach (int Process in SSEMI.Processes.ToList())
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