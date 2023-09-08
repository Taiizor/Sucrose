using SSECSMI = Sucrose.Shared.Engine.CefSharp.Manage.Internal;
using SWEACAM = Skylark.Wing.Extension.AudioController.AudioManager;
using SWEVPCAM = Skylark.Wing.Extension.VideoPlayerController.AudioManager;

namespace Sucrose.Shared.Engine.CefSharp.Helper
{
    internal static class Url
    {
        public static void SetVolume(int Volume)
        {
            //MessageBox.Show(SSECSMI.CefEngine.GetBrowser().GetHost().GetWindowHandle() + "-" + SSECSMI.ProcessId);

            try
            {
                SWEVPCAM.SetApplicationVolume(SSECSMI.ProcessId, Volume);
            }
            catch
            {
                SWEACAM.SetApplicationVolume(SSECSMI.ProcessId, Volume);
            }
        }
    }
}