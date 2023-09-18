using Sucrose.Grpc.Common;
using SGCSBCS = Sucrose.Grpc.Client.Services.BackgroundogClientService;
using SGSGSS = Sucrose.Grpc.Services.GeneralServerService;
using SMMM = Sucrose.Manager.Manage.Manager;
using SMR = Sucrose.Memory.Readonly;
using SSDECT = Sucrose.Shared.Dependency.Enum.CommandsType;
using SSEMI = Sucrose.Shared.Engine.Manage.Internal;
using SSEMM = Sucrose.Shared.Engine.Manage.Manager;
using SSSHP = Sucrose.Shared.Space.Helper.Processor;
using SSSMI = Sucrose.Shared.Space.Manage.Internal;

namespace Sucrose.Shared.Engine.Helper
{
    internal static class System
    {
        public static string GetSystemCpu()
        {
            if (SSSHP.Work(SMR.Backgroundog))
            {
                try
                {
                    SGSGSS.ChannelCreate($"{SSEMM.Host}", SSEMM.Port);
                    SSEMI.Client = new(SGSGSS.ChannelInstance);

                    BackgroundogCpuResponse Response = SGCSBCS.GetCpu(SSEMI.Client);

                    return Response.Info;
                }
                catch
                {
                    //
                }
            }
            else if (SMMM.PerformanceCounter)
            {
                SSSHP.Run(SSSMI.Commandog, $"{SMR.StartCommand}{SSDECT.Backgroundog}{SMR.ValueSeparator}{SSSMI.Backgroundog}");
            }

            return string.Empty;
        }

        public static string GetSystemDate()
        {
            if (SSSHP.Work(SMR.Backgroundog))
            {
                try
                {
                    SGSGSS.ChannelCreate($"{SSEMM.Host}", SSEMM.Port);
                    SSEMI.Client = new(SGSGSS.ChannelInstance);

                    BackgroundogDateResponse Response = SGCSBCS.GetDate(SSEMI.Client);

                    return Response.Info;
                }
                catch
                {
                    //
                }
            }
            else if (SMMM.PerformanceCounter)
            {
                SSSHP.Run(SSSMI.Commandog, $"{SMR.StartCommand}{SSDECT.Backgroundog}{SMR.ValueSeparator}{SSSMI.Backgroundog}");
            }

            return string.Empty;
        }

        public static string GetSystemMemory()
        {
            if (SSSHP.Work(SMR.Backgroundog))
            {
                try
                {
                    SGSGSS.ChannelCreate($"{SSEMM.Host}", SSEMM.Port);
                    SSEMI.Client = new(SGSGSS.ChannelInstance);

                    BackgroundogMemoryResponse Response = SGCSBCS.GetMemory(SSEMI.Client);

                    return Response.Info;
                }
                catch
                {
                    //
                }
            }
            else if (SMMM.PerformanceCounter)
            {
                SSSHP.Run(SSSMI.Commandog, $"{SMR.StartCommand}{SSDECT.Backgroundog}{SMR.ValueSeparator}{SSSMI.Backgroundog}");
            }

            return string.Empty;
        }

        public static string GetSystemBattery()
        {
            if (SSSHP.Work(SMR.Backgroundog))
            {
                try
                {
                    SGSGSS.ChannelCreate($"{SSEMM.Host}", SSEMM.Port);
                    SSEMI.Client = new(SGSGSS.ChannelInstance);

                    BackgroundogBatteryResponse Response = SGCSBCS.GetBattery(SSEMI.Client);

                    return Response.Info;
                }
                catch
                {
                    //
                }
            }
            else if (SMMM.PerformanceCounter)
            {
                SSSHP.Run(SSSMI.Commandog, $"{SMR.StartCommand}{SSDECT.Backgroundog}{SMR.ValueSeparator}{SSSMI.Backgroundog}");
            }

            return string.Empty;
        }

        public static string GetSystemNetwork()
        {
            if (SSSHP.Work(SMR.Backgroundog))
            {
                try
                {
                    SGSGSS.ChannelCreate($"{SSEMM.Host}", SSEMM.Port);
                    SSEMI.Client = new(SGSGSS.ChannelInstance);

                    BackgroundogNetworkResponse Response = SGCSBCS.GetNetwork(SSEMI.Client);

                    return Response.Info;
                }
                catch
                {
                    //
                }
            }
            else if (SMMM.PerformanceCounter)
            {
                SSSHP.Run(SSSMI.Commandog, $"{SMR.StartCommand}{SSDECT.Backgroundog}{SMR.ValueSeparator}{SSSMI.Backgroundog}");
            }

            return string.Empty;
        }

        public static string GetSystemMotherboard()
        {
            if (SSSHP.Work(SMR.Backgroundog))
            {
                try
                {
                    SGSGSS.ChannelCreate($"{SSEMM.Host}", SSEMM.Port);
                    SSEMI.Client = new(SGSGSS.ChannelInstance);

                    BackgroundogMotherboardResponse Response = SGCSBCS.GetMotherboard(SSEMI.Client);

                    return Response.Info;
                }
                catch
                {
                    //
                }
            }
            else if (SMMM.PerformanceCounter)
            {
                SSSHP.Run(SSSMI.Commandog, $"{SMR.StartCommand}{SSDECT.Backgroundog}{SMR.ValueSeparator}{SSSMI.Backgroundog}");
            }

            return string.Empty;
        }
    }
}