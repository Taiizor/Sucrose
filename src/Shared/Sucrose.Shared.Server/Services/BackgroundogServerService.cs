#if BACKGROUNDOG

using Google.Protobuf.WellKnownTypes;
using Grpc.Core;
using Newtonsoft.Json;
using Sucrose.Grpc.Common;
using static Sucrose.Grpc.Common.Backgroundog;
using SBED = Sucrose.Backgroundog.Extension.Data;

namespace Sucrose.Shared.Server.Services
{
    public class BackgroundogServerService : BackgroundogBase
    {
        public override Task<BackgroundogMotherboardResponse> MotherboardBackgroundog(Empty _, ServerCallContext Context)
        {
            return Task.FromResult(new BackgroundogMotherboardResponse { Info = JsonConvert.SerializeObject(SBED.GetMotherboardInfo(), Formatting.Indented) });
        }

        public override Task<BackgroundogNetworkResponse> NetworkBackgroundog(Empty _, ServerCallContext Context)
        {
            return Task.FromResult(new BackgroundogNetworkResponse { Info = JsonConvert.SerializeObject(SBED.GetNetworkInfo(), Formatting.Indented) });
        }

        public override Task<BackgroundogBatteryResponse> BatteryBackgroundog(Empty _, ServerCallContext Context)
        {
            return Task.FromResult(new BackgroundogBatteryResponse { Info = JsonConvert.SerializeObject(SBED.GetBatteryInfo(), Formatting.Indented) });
        }

        public override Task<BackgroundogMemoryResponse> MemoryBackgroundog(Empty _, ServerCallContext Context)
        {
            return Task.FromResult(new BackgroundogMemoryResponse { Info = JsonConvert.SerializeObject(SBED.GetMemoryInfo(), Formatting.Indented) });
        }

        public override Task<BackgroundogDateResponse> DateBackgroundog(Empty _, ServerCallContext Context)
        {
            return Task.FromResult(new BackgroundogDateResponse { Info = JsonConvert.SerializeObject(SBED.GetDateInfo(), Formatting.Indented) });
        }

        public override Task<BackgroundogCpuResponse> CpuBackgroundog(Empty _, ServerCallContext Context)
        {
            return Task.FromResult(new BackgroundogCpuResponse { Info = JsonConvert.SerializeObject(SBED.GetCpuInfo(), Formatting.Indented) });
        }
    }
}

#endif