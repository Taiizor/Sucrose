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
using System.Runtime.ExceptionServices;
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
            DispatcherUnhandledException += App_DispatcherUnhandledException;
            AppDomain.CurrentDomain.FirstChanceException += App_FirstChanceException;
            AppDomain.CurrentDomain.UnhandledException += App_GlobalUnhandledExceptionHandler;

#if NET48_OR_GREATER && DEBUG
            CefRuntime.SubscribeAnyCpuAssemblyResolver();
#endif

            CefSettings Settings = new()
            {
                CachePath = Path.Combine(Readonly.AppDataPath, Readonly.AppName, Readonly.CefSharp, Readonly.CacheFolder)
            };

            Settings.CefCommandLineArgs.Add("enable-media-stream");
            Settings.CefCommandLineArgs.Add("use-fake-ui-for-media-stream");
            Settings.CefCommandLineArgs.Add("enable-usermedia-screen-capturing");

            //Example of checking if a call to Cef.Initialize has already been made, we require this for
            //our .Net 5.0 Single File Publish example, you don't typically need to perform this check
            //if you call Cef.Initialze within your WPF App constructor.
            if (!Cef.IsInitialized)
            {
                //Perform dependency check to make sure all relevant resources are in our output directory.
                Cef.Initialize(Settings, performDependencyCheck: true, browserProcessHandler: null);
            }
        }

        protected override void OnStartup(StartupEventArgs e)
        {
            base.OnStartup(e);

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

            GeneralServerService.ServerInstance.KillAsync().Wait();
            //GeneralServerService.ServerInstance.ShutdownAsync().Wait();
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

        private void App_FirstChanceException(object sender, FirstChanceExceptionEventArgs e)
        {
            // Dispose işlemleri burada gerçekleştirilebilir
            // ...

            // İstisnayı işleyin veya loglayın
            //Exception exception = e.Exception;
            // ...

            // İstisnayı işledikten sonra uygulamayı kapatmak istiyorsanız aşağıdaki satırı ekleyebilirsiniz
            Shutdown();
        }

        private void App_GlobalUnhandledExceptionHandler(object sender, UnhandledExceptionEventArgs e)
        {
            // Dispose işlemleri burada gerçekleştirilebilir
            // ...

            // İstisnayı işleyin veya loglayın
            //Exception exception = (Exception)e.ExceptionObject;
            // ...

            // İstisnayı işledikten sonra uygulamayı kapatmak istiyorsanız aşağıdaki satırı ekleyebilirsiniz
            Shutdown();
        }
    }
}