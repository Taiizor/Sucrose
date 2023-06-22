using Grpc.Core;
using Sucrose.Common.Manage;
using Sucrose.Common.Services;
using Sucrose.Grpc.Common;
using Sucrose.Grpc.Services;
using Application = System.Windows.Application;

namespace Sucrose.WPF.CS
{
    /// <summary>
    /// Interaction logic for App.xaml
    /// </summary>
    public partial class App : Application
    {
        public App()
        {
            Internal.TrayIconManager.Start();

            GeneralServerService.ServerCreate(new List<ServerServiceDefinition>
            {
                Websiter.BindService(new WebsiterServerService()),
                Trayer.BindService(new TrayerServerService())
            });
        }
    }
}