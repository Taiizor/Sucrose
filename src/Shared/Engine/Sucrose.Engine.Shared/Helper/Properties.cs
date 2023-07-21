using SESHD = Sucrose.Engine.Shared.Helper.Data;
using SESMI = Sucrose.Engine.Shared.Manage.Internal;
using SESMIEN = Sucrose.Engine.Shared.Manage.Internal.ExecuteNormal;
using SESMIET = Sucrose.Engine.Shared.Manage.Internal.ExecuteTask;

namespace Sucrose.Engine.Shared.Helper
{
    internal static class Properties
    {
        public static void ExecuteNormal(SESMIEN Function)
        {
            if (SESMI.Initialized)
            {
                if (!string.IsNullOrEmpty(SESMI.Properties.LoopMode))
                {
                    Function(string.Format(SESMI.Properties.LoopMode, SESHD.GetLoop()));
                }

                if (!string.IsNullOrEmpty(SESMI.Properties.VolumeLevel))
                {
                    Function(string.Format(SESMI.Properties.VolumeLevel, SESHD.GetVolume()));
                }

                if (!string.IsNullOrEmpty(SESMI.Properties.ShuffleMode))
                {
                    Function(string.Format(SESMI.Properties.ShuffleMode, SESHD.GetShuffle()));
                }

                if (!string.IsNullOrEmpty(SESMI.Properties.StretchMode))
                {
                    Function(string.Format(SESMI.Properties.StretchMode, SESHD.GetStretch()));
                }

                if (!string.IsNullOrEmpty(SESMI.Properties.ComputerDate))
                {
                    Function(string.Format(SESMI.Properties.ComputerDate, SESHD.GetComputerDate()));
                }
            }
        }

        public static void ExecuteTask(SESMIET Function)
        {
            if (SESMI.Initialized)
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