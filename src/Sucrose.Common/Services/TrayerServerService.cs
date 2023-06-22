#if WPF_CS
    using Google.Protobuf.WellKnownTypes;
    using Grpc.Core;
    using Sucrose.Common.Manage;
    using Sucrose.Grpc.Common;
    using System.Threading.Tasks;
    using static Sucrose.Grpc.Common.Trayer;

    namespace Sucrose.Common.Services
    {
        public class TrayerServerService : TrayerBase
        {
            public override Task<TrayStateResponse> StateTray(Empty _, ServerCallContext Context)
            {
                return Task.FromResult(new TrayStateResponse { State = Internal.TrayIconManager.State() });
            }

            public override Task<TrayShowResponse> ShowTray(Empty _, ServerCallContext Context)
            {
                return Task.FromResult(new TrayShowResponse { Result = Internal.TrayIconManager.Show() });
            }

            public override Task<TrayHideResponse> HideTray(Empty _, ServerCallContext Context)
            {
                return Task.FromResult(new TrayHideResponse { Result = Internal.TrayIconManager.Hide() });
            }
        }
    }
#endif