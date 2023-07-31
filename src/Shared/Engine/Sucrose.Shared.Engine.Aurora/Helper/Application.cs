using SSEAMI = Sucrose.Shared.Engine.Aurora.Manage.Internal;
using SWEACAM = Skylark.Wing.Extension.AudioController.AudioManager;
using SWEVPCAM = Skylark.Wing.Extension.VideoPlayerController.AudioManager;

namespace Sucrose.Shared.Engine.Aurora.Helper
{
    internal static class Application
    {
        public static void SetVolume(int Volume)
        {
            try
            {
                SWEVPCAM.SetApplicationVolume(SSEAMI.ApplicationProcess.Id, Volume);
            }
            catch
            {
                SWEACAM.SetApplicationVolume(SSEAMI.ApplicationProcess.Id, Volume);
            }
        }
    }
}