using SESMIEN = Sucrose.Shared.Engine.Manage.Internal.ExecuteNormal;
using SESMIET = Sucrose.Shared.Engine.Manage.Internal.ExecuteTask;
using SSEHD = Sucrose.Shared.Engine.Helper.Data;
using SSEHS = Sucrose.Shared.Engine.Helper.System;
using SSEMI = Sucrose.Shared.Engine.Manage.Internal;

namespace Sucrose.Shared.Engine.Helper
{
    internal static class Compatible
    {
        public static async void ExecuteNormal(SESMIEN Function)
        {
            if (SSEMI.Initialized)
            {
                _ = Task.Run(SSEHS.GetSystem);

                if (!string.IsNullOrEmpty(SSEMI.Compatible.LoopMode))
                {
                    Function(string.Format(SSEMI.Compatible.LoopMode, SSEHD.GetLoop()));
                }

                if (!string.IsNullOrEmpty(SSEMI.Compatible.SystemCpu))
                {
                    Function(string.Format(SSEMI.Compatible.SystemCpu, SSEMI.CpuData));
                }

                if (!string.IsNullOrEmpty(SSEMI.Compatible.SystemBios))
                {
                    Function(string.Format(SSEMI.Compatible.SystemBios, SSEMI.BiosData));
                }

                if (!string.IsNullOrEmpty(SSEMI.Compatible.SystemDate))
                {
                    Function(string.Format(SSEMI.Compatible.SystemDate, SSEMI.DateData));
                }

                if (!string.IsNullOrEmpty(SSEMI.Compatible.SystemAudio))
                {
                    Function(string.Format(SSEMI.Compatible.SystemAudio, SSEMI.AudioData));
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
                    Function(string.Format(SSEMI.Compatible.SystemMemory, SSEMI.MemoryData));
                }

                if (!string.IsNullOrEmpty(SSEMI.Compatible.SystemBattery))
                {
                    Function(string.Format(SSEMI.Compatible.SystemBattery, SSEMI.BatteryData));
                }

                if (!string.IsNullOrEmpty(SSEMI.Compatible.SystemGraphic))
                {
                    Function(string.Format(SSEMI.Compatible.SystemGraphic, SSEMI.GraphicData));
                }

                if (!string.IsNullOrEmpty(SSEMI.Compatible.SystemNetwork))
                {
                    Function(string.Format(SSEMI.Compatible.SystemNetwork, SSEMI.NetworkData));
                }

                if (!string.IsNullOrEmpty(SSEMI.Compatible.SystemMotherboard))
                {
                    Function(string.Format(SSEMI.Compatible.SystemMotherboard, SSEMI.MotherboardData));
                }
            }

            await Task.CompletedTask;
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