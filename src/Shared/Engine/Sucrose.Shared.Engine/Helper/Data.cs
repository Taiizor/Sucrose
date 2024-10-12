using SEDEST = Skylark.Enum.DuplicateScreenType;
using SEDYST = Skylark.Enum.DisplayScreenType;
using SEEST = Skylark.Enum.ExpandScreenType;
using SEST = Skylark.Enum.ScreenType;
using SMMM = Sucrose.Manager.Manage.Manager;
using SSDEST = Sucrose.Shared.Dependency.Enum.StretchType;
using SSDMI = Sucrose.Shared.Dependency.Manage.Internal;
using SSDMME = Sucrose.Shared.Dependency.Manage.Manager.Engine;
using SSEMI = Sucrose.Shared.Engine.Manage.Internal;
using SWUD = Skylark.Wing.Utility.Desktop;
using SMME = Sucrose.Manager.Manage.Engine;
using SMMCE = Sucrose.Memory.Manage.Constant.Engine;
using SMMMCE = Sucrose.Memory.Manage.Constant.Engine;

namespace Sucrose.Shared.Engine.Helper
{
    internal static class Data
    {
        public static bool GetLoop()
        {
            return SMME.Loop;
        }

        public static int GetVolume()
        {
            if (SMME.Volume > 0)
            {
                if (SMME.VolumeActive)
                {
                    if (SSEMI.PauseVolume)
                    {
                        return 0;
                    }
                }

                if (SMME.VolumeDesktop)
                {
                    if (SWUD.IsDesktopBasic() || SWUD.IsDesktopAdvanced())
                    {
                        return SMME.Volume;
                    }
                    else
                    {
                        return 0;
                    }
                }
            }

            return SMME.Volume;
        }

        public static bool GetShuffle()
        {
            return SMME.Shuffle;
        }

        public static SSDEST GetStretch()
        {
            SSDEST Stretch = SSDMME.StretchType;

            if ((int)Stretch < Enum.GetValues(typeof(SSDEST)).Length)
            {
                return Stretch;
            }
            else
            {
                return SSDMI.DefaultStretchType;
            }
        }

        public static SEST GetScreenType()
        {
            return SMME.ScreenType;
        }

        public static int GetScreenIndex()
        {
            return SMME.ScreenIndex;
        }

        public static SEEST GetExpandScreenType()
        {
            return SMME.ExpandScreenType;
        }

        public static SEDYST GetDisplayScreenType()
        {
            return SMME.DisplayScreenType;
        }

        public static SEDEST GetDuplicateScreenType()
        {
            return SMME.DuplicateScreenType;
        }
    }
}