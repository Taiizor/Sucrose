using Google.Protobuf.WellKnownTypes;
using Sucrose.Grpc.Common;
using static Sucrose.Grpc.Common.TrayIcon;

namespace Sucrose.Grpc.Client.Services
{
    public static class TrayIconClientService
    {
        public static TrayDisposeResponse GetDispose(TrayIconClient Client)
        {
            TrayDisposeResponse Response = Client.DisposeTray(new Empty());

            return Response;
        }

        public static TrayStateResponse GetState(TrayIconClient Client)
        {
            TrayStateResponse Response = Client.StateTray(new Empty());

            return Response;
        }

        public static TrayShowResponse GetShow(TrayIconClient Client)
        {
            TrayShowResponse Response = Client.ShowTray(new Empty());

            return Response;
        }

        public static TrayHideResponse GetHide(TrayIconClient Client)
        {
            TrayHideResponse Response = Client.HideTray(new Empty());

            return Response;
        }
    }
}