#if LAUNCHER

using Google.Protobuf.WellKnownTypes;
using Grpc.Core;
using Sucrose.Grpc.Common;
using SSLMI = Sucrose.Shared.Launcher.Manage.Internal;
using static Sucrose.Grpc.Common.Launcher;

namespace Sucrose.Shared.Server.Services
{
    public class LauncherServerService : LauncherBase
    {
        public override Task<LauncherReleaseResponse> ReleaseLauncher(Empty _, ServerCallContext Context)
        {
            return Task.FromResult(new LauncherReleaseResponse { State = SSLMI.TrayIconManager.Release() });
        }

        public override Task<LauncherStateResponse> StateLauncher(Empty _, ServerCallContext Context)
        {
            return Task.FromResult(new LauncherStateResponse { State = SSLMI.TrayIconManager.State() });
        }

        public override Task<LauncherShowResponse> ShowLauncher(Empty _, ServerCallContext Context)
        {
            return Task.FromResult(new LauncherShowResponse { Result = SSLMI.TrayIconManager.Show() });
        }

        public override Task<LauncherHideResponse> HideLauncher(Empty _, ServerCallContext Context)
        {
            return Task.FromResult(new LauncherHideResponse { Result = SSLMI.TrayIconManager.Hide() });
        }
    }
}

#endif