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
                    SGSGSS.ChannelCreate($"{SSEMM.Host}", SSEMM.Port);
                    SSEMI.Client = new(SGSGSS.ChannelInstance);
                }
                catch { }

                _ = Task.Run(() =>
                {
                    try
                    {
                        BackgroundogCpuResponse Response = SGCSBCS.GetCpu(SSEMI.Client);
                        SSEMI.CpuData = Response.Info;
                    }
                    catch
                    {
                        SSEMI.CpuData = string.Empty;
                    }
                });

                _ = Task.Run(() =>
                {
                    try
                    {
                        BackgroundogBiosResponse Response = SGCSBCS.GetBios(SSEMI.Client);
                        SSEMI.BiosData = Response.Info;
                    }
                    catch
                    {
                        SSEMI.BiosData = string.Empty;
                    }
                });

                _ = Task.Run(() =>
                {
                    try
                    {
                        BackgroundogDateResponse Response = SGCSBCS.GetDate(SSEMI.Client);
                        SSEMI.DateData = Response.Info;
                    }
                    catch
                    {
                        SSEMI.DateData = string.Empty;
                    }
                });

                _ = Task.Run(() =>
                {
                    try
                    {
                        BackgroundogAudioResponse Response = SGCSBCS.GetAudio(SSEMI.Client);
                        SSEMI.AudioData = Response.Info;
                    }
                    catch
                    {
                        SSEMI.AudioData = string.Empty;
                    }
                });

                _ = Task.Run(() =>
                {
                    try
                    {
                        BackgroundogMemoryResponse Response = SGCSBCS.GetMemory(SSEMI.Client);
                        SSEMI.MemoryData = Response.Info;
                    }
                    catch
                    {
                        SSEMI.MemoryData = string.Empty;
                    }
                });

                _ = Task.Run(() =>
                {
                    try
                    {
                        BackgroundogBatteryResponse Response = SGCSBCS.GetBattery(SSEMI.Client);
                        SSEMI.BatteryData = Response.Info;
                    }
                    catch
                    {
                        SSEMI.BatteryData = string.Empty;
                    }
                });

                _ = Task.Run(() =>
                {
                    try
                    {
                        BackgroundogGraphicResponse Response = SGCSBCS.GetGraphic(SSEMI.Client);
                        SSEMI.GraphicData = Response.Info;
                    }
                    catch
                    {
                        SSEMI.GraphicData = string.Empty;
                    }
                });

                _ = Task.Run(() =>
                {
                    try
                    {
                        BackgroundogNetworkResponse Response = SGCSBCS.GetNetwork(SSEMI.Client);
                        SSEMI.NetworkData = Response.Info;
                    }
                    catch
                    {
                        SSEMI.NetworkData = string.Empty;
                    }
                });

                _ = Task.Run(() =>
                {
                    try
                    {
                        BackgroundogMotherboardResponse Response = SGCSBCS.GetMotherboard(SSEMI.Client);
                        SSEMI.MotherboardData = Response.Info;
                    }
                    catch
                    {
                        SSEMI.MotherboardData = string.Empty;
                    }
                });
            }
            else
            {
                SSEMI.CpuData = string.Empty;
                SSEMI.BiosData = string.Empty;
                SSEMI.DateData = string.Empty;
                SSEMI.AudioData = string.Empty;
                SSEMI.MemoryData = string.Empty;
                SSEMI.BatteryData = string.Empty;
                SSEMI.GraphicData = string.Empty;
                SSEMI.NetworkData = string.Empty;
                SSEMI.MotherboardData = string.Empty;
            }

            await Task.CompletedTask;
        }
    }
}