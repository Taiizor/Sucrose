#if BACKGROUNDOG

using Google.Protobuf.WellKnownTypes;
using Grpc.Core;
using Sucrose.Grpc.Common;
using static Sucrose.Grpc.Common.Backgroundog;
using SBMI = Sucrose.Backgroundog.Manage.Internal;

namespace Sucrose.Shared.Server.Services
{
    public class BackgroundogServerService : BackgroundogBase
    {
        public override Task<BackgroundogStateResponse> StateBackgroundog(Empty _, ServerCallContext Context)
        {
            return Task.FromResult(new BackgroundogStateResponse { State = SBMI.State });
        }
    }
}

#endif