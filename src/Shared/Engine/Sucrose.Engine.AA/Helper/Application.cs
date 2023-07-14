using SEAAMI = Sucrose.Engine.AA.Manage.Internal;
using SWHACAM = Skylark.Wing.Helper.AudioController.AudioManager;
using SWHVPCAM = Skylark.Wing.Helper.VideoPlayerController.AudioManager;

namespace Sucrose.Engine.AA.Helper
{
    internal static class Application
    {
        public static void SetVolume(int Volume)
        {
            try
            {
                SWHVPCAM.SetApplicationVolume(SEAAMI.ApplicationProcess.Id, Volume);
            }
            catch
            {
                SWHACAM.SetApplicationVolume(SEAAMI.ApplicationProcess.Id, Volume);
            }
        }
    }
}