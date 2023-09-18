using SSEMM = Sucrose.Shared.Engine.Manage.Manager;
using SSEMI = Sucrose.Shared.Engine.Manage.Internal;
using SGSGSS = Sucrose.Grpc.Services.GeneralServerService;
using SGCSBCS = Sucrose.Grpc.Client.Services.BackgroundogClientService;
using SMR = Sucrose.Memory.Readonly;
using SSSHP = Sucrose.Shared.Space.Helper.Processor;
using Sucrose.Grpc.Common;

namespace Sucrose.Shared.Engine.Helper
{
    internal static class System
    {
        public static string GetSystemCpu()
        {
            if (SSSHP.Work(SMR.Backgroundog))
            {
                SGSGSS.ChannelCreate($"{SSEMM.Host}", SSEMM.Port);
                SSEMI.Client = new(SGSGSS.ChannelInstance);

                BackgroundogCpuResponse Response = SGCSBCS.GetCpu(SSEMI.Client);

                return Response.Info;
            }

            return string.Empty;
        }

        public static string GetSystemDate()
        {
            if (SSSHP.Work(SMR.Backgroundog))
            {
                SGSGSS.ChannelCreate($"{SSEMM.Host}", SSEMM.Port);
                SSEMI.Client = new(SGSGSS.ChannelInstance);

                BackgroundogDateResponse Response = SGCSBCS.GetDate(SSEMI.Client);

                return Response.Info;
            }

            return string.Empty;
        }

        public static string GetSystemMemory()
        {
            if (SSSHP.Work(SMR.Backgroundog))
            {
                SGSGSS.ChannelCreate($"{SSEMM.Host}", SSEMM.Port);
                SSEMI.Client = new(SGSGSS.ChannelInstance);

                BackgroundogMemoryResponse Response = SGCSBCS.GetMemory(SSEMI.Client);

                return Response.Info;
            }

            return string.Empty;
        }

        public static string GetSystemBattery()
        {
            if (SSSHP.Work(SMR.Backgroundog))
            {
                SGSGSS.ChannelCreate($"{SSEMM.Host}", SSEMM.Port);
                SSEMI.Client = new(SGSGSS.ChannelInstance);

                BackgroundogBatteryResponse Response = SGCSBCS.GetBattery(SSEMI.Client);

                return Response.Info;
            }

            return string.Empty;
        }

        public static string GetSystemNetwork()
        {
            if (SSSHP.Work(SMR.Backgroundog))
            {
                SGSGSS.ChannelCreate($"{SSEMM.Host}", SSEMM.Port);
                SSEMI.Client = new(SGSGSS.ChannelInstance);

                BackgroundogNetworkResponse Response = SGCSBCS.GetNetwork(SSEMI.Client);

                return Response.Info;
            }

            return string.Empty;
        }

        public static string GetSystemMotherboard()
        {
            if (SSSHP.Work(SMR.Backgroundog))
            {
                SGSGSS.ChannelCreate($"{SSEMM.Host}", SSEMM.Port);
                SSEMI.Client = new(SGSGSS.ChannelInstance);

                BackgroundogMotherboardResponse Response = SGCSBCS.GetMotherboard(SSEMI.Client);

                return Response.Info;
            }

            return string.Empty;
        }
    }
}