using SESMIEN = Sucrose.Shared.Engine.Manage.Internal.ExecuteNormal;
using SESMIET = Sucrose.Shared.Engine.Manage.Internal.ExecuteTask;
using SSEHD = Sucrose.Shared.Engine.Helper.Data;
using SSEHS = Sucrose.Shared.Engine.Helper.System;
using SSEMI = Sucrose.Shared.Engine.Manage.Internal;

namespace Sucrose.Shared.Engine.Helper
{
    internal static class Compatible
    {
        public static void ExecuteNormal(SESMIEN Function)
        {
            if (SSEMI.Initialized)
            {
                if (!string.IsNullOrEmpty(SSEMI.Compatible.LoopMode))
                {
                    Function(string.Format(SSEMI.Compatible.LoopMode, SSEHD.GetLoop()));
                }

                if (!string.IsNullOrEmpty(SSEMI.Compatible.SystemCpu))
                {
                    Function(string.Format(SSEMI.Compatible.SystemCpu, SSEHS.GetSystemCpu()));
                }

                if (!string.IsNullOrEmpty(SSEMI.Compatible.SystemBios))
                {
                    Function(string.Format(SSEMI.Compatible.SystemBios, SSEHS.GetSystemBios()));
                }

                if (!string.IsNullOrEmpty(SSEMI.Compatible.SystemDate))
                {
                    Function(string.Format(SSEMI.Compatible.SystemDate, SSEHS.GetSystemDate()));
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

                if (!string.IsNullOrEmpty(SSEMI.Compatible.SystemMemory))
                {
                    Function(string.Format(SSEMI.Compatible.SystemMemory, SSEHS.GetSystemMemory()));
                }

                if (!string.IsNullOrEmpty(SSEMI.Compatible.SystemBattery))
                {
                    Function(string.Format(SSEMI.Compatible.SystemBattery, SSEHS.GetSystemBattery()));
                }

                if (!string.IsNullOrEmpty(SSEMI.Compatible.SystemNetwork))
                {
                    Function(string.Format(SSEMI.Compatible.SystemNetwork, SSEHS.GetSystemNetwork()));
                }

                if (!string.IsNullOrEmpty(SSEMI.Compatible.SystemMotherboard))
                {
                    Function(string.Format(SSEMI.Compatible.SystemMotherboard, SSEHS.GetSystemMotherboard()));
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