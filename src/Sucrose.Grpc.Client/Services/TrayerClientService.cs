using Google.Protobuf.WellKnownTypes;
using Sucrose.Grpc.Common;
using static Sucrose.Grpc.Common.Trayer;

namespace Sucrose.Grpc.Client.Services
{
    public static class TrayerClientService
    {
        public static TrayStateResponse GetState(TrayerClient Client)
        {
            TrayStateResponse Response = Client.StateTray(new Empty());

            return Response;
        }

        public static TrayShowResponse GetShow(TrayerClient Client)
        {
            TrayShowResponse Response = Client.ShowTray(new Empty());

            return Response;
        }

        public static TrayHideResponse GetHide(TrayerClient Client)
        {
            TrayHideResponse Response = Client.HideTray(new Empty());

            return Response;
        }
    }
}