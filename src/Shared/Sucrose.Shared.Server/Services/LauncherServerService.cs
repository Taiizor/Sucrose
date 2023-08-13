#if LAUNCHER

using Google.Protobuf.WellKnownTypes;
using Grpc.Core;
using Sucrose.Grpc.Common;
using Sucrose.Shared.Launcher.Manage;
using static Sucrose.Grpc.Common.Launcher;

namespace Sucrose.Shared.Server.Services
{
    public class LauncherServerService : LauncherBase
    {
        public override Task<LauncherReleaseResponse> ReleaseLauncher(Empty _, ServerCallContext Context)
        {
            return Task.FromResult(new LauncherReleaseResponse { State = Internal.TrayIconManager.Release() });
        }

        public override Task<LauncherStateResponse> StateLauncher(Empty _, ServerCallContext Context)
        {
            return Task.FromResult(new LauncherStateResponse { State = Internal.TrayIconManager.State() });
        }

        public override Task<LauncherShowResponse> ShowLauncher(Empty _, ServerCallContext Context)
        {
            return Task.FromResult(new LauncherShowResponse { Result = Internal.TrayIconManager.Show() });
        }

        public override Task<LauncherHideResponse> HideLauncher(Empty _, ServerCallContext Context)
        {
            return Task.FromResult(new LauncherHideResponse { Result = Internal.TrayIconManager.Hide() });
        }
    }
}

#endif