using SESMIEN = Sucrose.Shared.Engine.Manage.Internal.ExecuteNormal;
using SESMIET = Sucrose.Shared.Engine.Manage.Internal.ExecuteTask;
using SSEHD = Sucrose.Shared.Engine.Helper.Data;
using SSEMI = Sucrose.Shared.Engine.Manage.Internal;

namespace Sucrose.Shared.Engine.Helper
{
    internal static class Properties
    {
        public static void ExecuteNormal(SESMIEN Function)
        {
            if (SSEMI.Initialized)
            {
                if (!string.IsNullOrEmpty(SSEMI.Properties.LoopMode))
                {
                    Function(string.Format(SSEMI.Properties.LoopMode, SSEHD.GetLoop()));
                }

                if (!string.IsNullOrEmpty(SSEMI.Properties.VolumeLevel))
                {
                    Function(string.Format(SSEMI.Properties.VolumeLevel, SSEHD.GetVolume()));
                }

                if (!string.IsNullOrEmpty(SSEMI.Properties.ShuffleMode))
                {
                    Function(string.Format(SSEMI.Properties.ShuffleMode, SSEHD.GetShuffle()));
                }

                if (!string.IsNullOrEmpty(SSEMI.Properties.StretchMode))
                {
                    Function(string.Format(SSEMI.Properties.StretchMode, SSEHD.GetStretch()));
                }

                if (!string.IsNullOrEmpty(SSEMI.Properties.ComputerDate))
                {
                    Function(string.Format(SSEMI.Properties.ComputerDate, SSEHD.GetComputerDate()));
                }
            }
        }

        public static void ExecuteTask(SESMIET Function)
        {
            if (SSEMI.Initialized)
            {
                SESMIEN AdaptedFunction = new(async (Script) =>
                {
                    await Function(Script);
                });

                ExecuteNormal(AdaptedFunction);
            }
        }
    }
}