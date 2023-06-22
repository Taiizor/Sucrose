using Grpc.Core;
using Sucrose.Common.Manage;
using Sucrose.Common.Services;
using Sucrose.Grpc.Common;
using Sucrose.Grpc.Services;
using System.Windows;
using System.Windows.Threading;
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

        protected override void OnStartup(StartupEventArgs e)
        {
            base.OnStartup(e);

            // DispatcherUnhandledException olayına bir olay işleyici ekleyin
            DispatcherUnhandledException += App_DispatcherUnhandledException;
        }

        protected override void OnExit(ExitEventArgs e)
        {
            GeneralServerService.ServerInstance.ShutdownAsync().Wait();

            base.OnExit(e);
        }

        private void App_DispatcherUnhandledException(object sender, DispatcherUnhandledExceptionEventArgs e)
        {
            // Dispose işlemleri burada gerçekleştirilebilir
            // ...

            // İstisnayı işleyin veya loglayın
            //Exception exception = e.Exception;
            // ...

            // İstisnayı işledikten sonra uygulamayı kapatmak istiyorsanız aşağıdaki satırı ekleyebilirsiniz
            Shutdown();
        }
    }
}