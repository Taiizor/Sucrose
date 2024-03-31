using SEDST = Skylark.Enum.DuplicateScreenType;
using SEEST = Skylark.Enum.ExpandScreenType;
using SEST = Skylark.Enum.ScreenType;
using SMMM = Sucrose.Manager.Manage.Manager;
using SSDEDT = Sucrose.Shared.Dependency.Enum.DisplayType;
using SSDEST = Sucrose.Shared.Dependency.Enum.StretchType;
using SSEMM = Sucrose.Shared.Engine.Manage.Manager;
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
            SSDEST Stretch = SSEMM.StretchType;

            if ((int)Stretch < Enum.GetValues(typeof(SSDEST)).Length)
            {
                return Stretch;
            }
            else
            {
                return SSEMM.DefaultStretchType;
            }
        }

        public static SEST GetScreenType()
        {
            return SSEMM.ScreenType;
        }

        public static int GetScreenIndex()
        {
            return SMMM.ScreenIndex;
        }

        public static SSDEDT GetDisplayType()
        {
            return SSEMM.DisplayType;
        }

        public static SEEST GetExpandScreenType()
        {
            return SSEMM.ExpandScreenType;
        }

        public static SEDST GetDuplicateScreenType()
        {
            return SSEMM.DuplicateScreenType;
        }
    }
}