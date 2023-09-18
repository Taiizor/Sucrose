using Google.Protobuf.WellKnownTypes;
using Sucrose.Grpc.Common;
using static Sucrose.Grpc.Common.Backgroundog;

namespace Sucrose.Grpc.Client.Services
{
    public static class BackgroundogClientService
    {
        public static BackgroundogStateResponse GetState(BackgroundogClient Client)
        {
            BackgroundogStateResponse Response = Client.StateBackgroundog(new Empty());

            return Response;
        }
    }
}