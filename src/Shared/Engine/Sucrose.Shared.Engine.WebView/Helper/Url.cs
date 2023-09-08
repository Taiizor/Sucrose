using SSEWVMI = Sucrose.Shared.Engine.WebView.Manage.Internal;
using SWEACAM = Skylark.Wing.Extension.AudioController.AudioManager;
using SWEVPCAM = Skylark.Wing.Extension.VideoPlayerController.AudioManager;

namespace Sucrose.Shared.Engine.WebView.Helper
{
    internal static class Url
    {
        public static void SetVolume(int Volume)
        {
            //MessageBox.Show(SSEWVMI.WebEngine.CoreWebView2.BrowserProcessId + "-" + SSEWVMI.ProcessId);

            try
            {
                SWEVPCAM.SetApplicationVolume(SSEWVMI.ProcessId, Volume);
            }
            catch
            {
                SWEACAM.SetApplicationVolume(SSEWVMI.ProcessId, Volume);
            }
        }
    }
}