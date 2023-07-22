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
                if (!string.IsNullOrEmpty(SSEMI.Compatible.LoopMode))
                {
                    Function(string.Format(SSEMI.Compatible.LoopMode, SSEHD.GetLoop()));
                }

                if (!string.IsNullOrEmpty(SSEMI.Compatible.VolumeLevel))
                {
                    Function(string.Format(SSEMI.Compatible.VolumeLevel, SSEHD.GetVolume()));
                }

                if (!string.IsNullOrEmpty(SSEMI.Compatible.ShuffleMode))
                {
                    Function(string.Format(SSEMI.Compatible.ShuffleMode, SSEHD.GetShuffle()));
                }

                if (!string.IsNullOrEmpty(SSEMI.Compatible.StretchMode))
                {
                    Function(string.Format(SSEMI.Compatible.StretchMode, SSEHD.GetStretch()));
                }

                if (!string.IsNullOrEmpty(SSEMI.Compatible.ComputerDate))
                {
                    Function(string.Format(SSEMI.Compatible.ComputerDate, SSEHD.GetComputerDate()));
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