using Sucrose.Grpc.Common;
using SGCSBCS = Sucrose.Grpc.Client.Services.BackgroundogClientService;
using SGSGSS = Sucrose.Grpc.Services.GeneralServerService;
using SMR = Sucrose.Memory.Readonly;
using SSEMI = Sucrose.Shared.Engine.Manage.Internal;
using SSEMM = Sucrose.Shared.Engine.Manage.Manager;
using SSSHP = Sucrose.Shared.Space.Helper.Processor;

namespace Sucrose.Shared.Engine.Helper
{
    internal static class System
    {
        public static async void GetSystem()
        {
            if (SSSHP.Work(SMR.Backgroundog))
            {
                try
                {
                    if (SSEMI.Client == null)
                    {
                        SGSGSS.ChannelCreate($"{SSEMM.Host}", SSEMM.Port);
                        SSEMI.Client = new(SGSGSS.ChannelInstance);
                    }

                    _ = Task.Run(() =>
                    {
                        BackgroundogCpuResponse Response = SGCSBCS.GetCpu(SSEMI.Client);
                        SSEMI.CpuData = Response.Info;
                    });

                    _ = Task.Run(() =>
                    {
                        BackgroundogBiosResponse Response = SGCSBCS.GetBios(SSEMI.Client);
                        SSEMI.BiosData = Response.Info;
                    });

                    _ = Task.Run(() =>
                    {
                        BackgroundogDateResponse Response = SGCSBCS.GetDate(SSEMI.Client);
                        SSEMI.DateData = Response.Info;
                    });

                    _ = Task.Run(() =>
                    {
                        BackgroundogAudioResponse Response = SGCSBCS.GetAudio(SSEMI.Client);
                        SSEMI.AudioData = Response.Info;
                    });

                    _ = Task.Run(() =>
                    {
                        BackgroundogMemoryResponse Response = SGCSBCS.GetMemory(SSEMI.Client);
                        SSEMI.MemoryData = Response.Info;
                    });

                    _ = Task.Run(() =>
                    {
                        BackgroundogBatteryResponse Response = SGCSBCS.GetBattery(SSEMI.Client);
                        SSEMI.BatteryData = Response.Info;
                    });

                    _ = Task.Run(() =>
                    {
                        BackgroundogNetworkResponse Response = SGCSBCS.GetNetwork(SSEMI.Client);
                        SSEMI.NetworkData = Response.Info;
                    });

                    _ = Task.Run(() =>
                    {
                        BackgroundogMotherboardResponse Response = SGCSBCS.GetMotherboard(SSEMI.Client);
                        SSEMI.MotherboardData = Response.Info;
                    });
                }
                catch
                {
                    //
                }
            }

            await Task.CompletedTask;
        }
    }
}