using SSEAMI = Sucrose.Shared.Engine.Aurora.Manage.Internal;
using SWHACAM = Skylark.Wing.Helper.AudioController.AudioManager;
using SWHVPCAM = Skylark.Wing.Helper.VideoPlayerController.AudioManager;

namespace Sucrose.Shared.Engine.Aurora.Helper
{
    internal static class Application
    {
        public static void SetVolume(int Volume)
        {
            try
            {
                SWHVPCAM.SetApplicationVolume(SSEAMI.ApplicationProcess.Id, Volume);
            }
            catch
            {
                SWHACAM.SetApplicationVolume(SSEAMI.ApplicationProcess.Id, Volume);
            }
        }
    }
}