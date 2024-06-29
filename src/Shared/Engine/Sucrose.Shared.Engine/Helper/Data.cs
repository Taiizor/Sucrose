using SEDEST = Skylark.Enum.DuplicateScreenType;
using SEDYST = Skylark.Enum.DisplayScreenType;
using SEEST = Skylark.Enum.ExpandScreenType;
using SEST = Skylark.Enum.ScreenType;
using SMMM = Sucrose.Manager.Manage.Manager;
using SSDEST = Sucrose.Shared.Dependency.Enum.StretchType;
using SSDMM = Sucrose.Shared.Dependency.Manage.Manager;
using SSEMI = Sucrose.Shared.Engine.Manage.Internal;
using SWUD = Skylark.Wing.Utility.Desktop;

namespace Sucrose.Shared.Engine.Helper
{
    internal static class Data
    {
        public static bool GetLoop()
        {
            return SMMM.Loop;
        }

        public static int GetVolume()
        {
            if (SMMM.VolumeActive)
            {
                if (SSEMI.PauseVolume)
                {
                    return 0;
                }
            }

            if (SMMM.VolumeDesktop)
            {
                if (SWUD.IsDesktopBasic() || SWUD.IsDesktopAdvanced())
                {
                    return SMMM.Volume;
                }
                else
                {
                    return 0;
                }
            }

            return SMMM.Volume;
        }

        public static bool GetShuffle()
        {
            return SMMM.Shuffle;
        }

        public static SSDEST GetStretch()
        {
            SSDEST Stretch = SSDMM.StretchType;

            if ((int)Stretch < Enum.GetValues(typeof(SSDEST)).Length)
            {
                return Stretch;
            }
            else
            {
                return SSDMM.DefaultStretchType;
            }
        }

        public static SEST GetScreenType()
        {
            return SMMM.ScreenType;
        }

        public static int GetScreenIndex()
        {
            return SMMM.ScreenIndex;
        }

        public static SEEST GetExpandScreenType()
        {
            return SMMM.ExpandScreenType;
        }

        public static SEDYST GetDisplayScreenType()
        {
            return SMMM.DisplayScreenType;
        }

        public static SEDEST GetDuplicateScreenType()
        {
            return SMMM.DuplicateScreenType;
        }
    }
}