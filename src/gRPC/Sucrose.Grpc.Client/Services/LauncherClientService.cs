using Google.Protobuf.WellKnownTypes;
using Sucrose.Grpc.Common;
using static Sucrose.Grpc.Common.Launcher;

namespace Sucrose.Grpc.Client.Services
{
    public static class LauncherClientService
    {
        public static LauncherReleaseResponse GetRelease(LauncherClient Client)
        {
            LauncherReleaseResponse Response = Client.ReleaseLauncher(new Empty());

            return Response;
        }

        public static LauncherStateResponse GetState(LauncherClient Client)
        {
            LauncherStateResponse Response = Client.StateLauncher(new Empty());

            return Response;
        }

        public static LauncherShowResponse GetShow(LauncherClient Client)
        {
            LauncherShowResponse Response = Client.ShowLauncher(new Empty());

            return Response;
        }

        public static LauncherHideResponse GetHide(LauncherClient Client)
        {
            LauncherHideResponse Response = Client.HideLauncher(new Empty());

            return Response;
        }
    }
}