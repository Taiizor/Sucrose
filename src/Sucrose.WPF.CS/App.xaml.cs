using CefSharp;
using CefSharp.Wpf;
using Grpc.Core;
using Skylark.Wing.Helper;
using Sucrose.Common.Manage;
using Sucrose.Common.Services;
using Sucrose.Grpc.Common;
using Sucrose.Grpc.Services;
using Sucrose.Memory;
using System.IO;
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
            CefRuntime.SubscribeAnyCpuAssemblyResolver();

            CefSettings settings = new()
            {
                CachePath = Path.Combine(Readonly.AppDataPath, Readonly.AppName, Readonly.CefSharp, Readonly.CacheFolder)
            };

            settings.CefCommandLineArgs.Add("enable-media-stream");
            settings.CefCommandLineArgs.Add("use-fake-ui-for-media-stream");
            settings.CefCommandLineArgs.Add("enable-usermedia-screen-capturing");

            //Example of checking if a call to Cef.Initialize has already been made, we require this for
            //our .Net 5.0 Single File Publish example, you don't typically need to perform this check
            //if you call Cef.Initialze within your WPF App constructor.
            if (!Cef.IsInitialized)
            {
                //Perform dependency check to make sure all relevant resources are in our output directory.
                Cef.Initialize(settings, performDependencyCheck: true, browserProcessHandler: null);
            }
        }

        protected override void OnStartup(StartupEventArgs e)
        {
            base.OnStartup(e);

            // DispatcherUnhandledException olayına bir olay işleyici ekleyin
            DispatcherUnhandledException += App_DispatcherUnhandledException;

            Internal.TrayIconManager.StartWPF(Current, WindowsTheme.GetTheme());

            GeneralServerService.ServerCreate(new List<ServerServiceDefinition>
            {
                Websiter.BindService(new WebsiterServerService()),
                Trayer.BindService(new TrayerServerService())
            });

            Internal.ServerManager.SetSetting("Host", GeneralServerService.Host);
            Internal.ServerManager.SetSetting("Port", GeneralServerService.Port);

            GeneralServerService.ServerInstance.Start();

            Main Browser = new();
            Browser.ShowDialog();

            Cef.Shutdown();
            Cef.PreShutdown();

            GeneralServerService.ServerInstance.ShutdownAsync().Wait();
        }

        protected override void OnExit(ExitEventArgs e)
        {
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