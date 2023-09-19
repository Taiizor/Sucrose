using Google.Protobuf.WellKnownTypes;
using Sucrose.Grpc.Common;
using static Sucrose.Grpc.Common.Backgroundog;

namespace Sucrose.Grpc.Client.Services
{
    public static class BackgroundogClientService
    {
        public static BackgroundogMotherboardResponse GetMotherboard(BackgroundogClient Client)
        {
            BackgroundogMotherboardResponse Response = Client.MotherboardBackgroundog(new Empty());

            return Response;
        }

        public static BackgroundogNetworkResponse GetNetwork(BackgroundogClient Client)
        {
            BackgroundogNetworkResponse Response = Client.NetworkBackgroundog(new Empty());

            return Response;
        }

        public static BackgroundogBatteryResponse GetBattery(BackgroundogClient Client)
        {
            BackgroundogBatteryResponse Response = Client.BatteryBackgroundog(new Empty());

            return Response;
        }

        public static BackgroundogMemoryResponse GetMemory(BackgroundogClient Client)
        {
            BackgroundogMemoryResponse Response = Client.MemoryBackgroundog(new Empty());

            return Response;
        }

        public static BackgroundogAudioResponse GetAudio(BackgroundogClient Client)
        {
            BackgroundogAudioResponse Response = Client.AudioBackgroundog(new Empty());

            return Response;
        }

        public static BackgroundogDateResponse GetDate(BackgroundogClient Client)
        {
            BackgroundogDateResponse Response = Client.DateBackgroundog(new Empty());

            return Response;
        }

        public static BackgroundogBiosResponse GetBios(BackgroundogClient Client)
        {
            BackgroundogBiosResponse Response = Client.BiosBackgroundog(new Empty());

            return Response;
        }

        public static BackgroundogCpuResponse GetCpu(BackgroundogClient Client)
        {
            BackgroundogCpuResponse Response = Client.CpuBackgroundog(new Empty());

            return Response;
        }
    }
}