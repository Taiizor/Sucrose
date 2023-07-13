using SEAAMI = Sucrose.Engine.AA.Manage.Internal;
using SWNM = Skylark.Wing.Native.Methods;
using SWHWAPI = Skylark.Wing.Helper.WinAPI;

namespace Sucrose.Engine.AA.Helper
{
    internal static class Application
    {
        public static void SetVolume(int Volume)
        {
            int NewVolume = (Volume * SWNM.MAX_VOLUME) / 100;

            SWHWAPI.waveOutSetVolume(SEAAMI.ApplicationHandle, (uint)((NewVolume << 16) | NewVolume));
        }
    }
}