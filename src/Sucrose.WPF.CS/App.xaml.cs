using CefSharp;
using CefSharp.Wpf;
using Grpc.Core;
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
        private static readonly Mutex Mutex = new(true, Readonly.CefSharpMutex);

        public App()
        {
            AppDomain.CurrentDomain.UnhandledException += GlobalUnhandledExceptionHandler;

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

        protected void Close()
        {
            Environment.Exit(0);
            Current.Shutdown();
            Shutdown();
        }

        protected override void OnExit(ExitEventArgs e)
        {
            base.OnExit(e);

            GeneralServerService.ServerInstance.KillAsync().Wait();
            //GeneralServerService.ServerInstance.ShutdownAsync().Wait();

            Close();
        }

        protected override void OnStartup(StartupEventArgs e)
        {
            base.OnStartup(e);

            GeneralServerService.ServerCreate(new List<ServerServiceDefinition>
            {
                Websiter.BindService(new WebsiterServerService())
            });

            Internal.ServerManager.SetSetting("Host", GeneralServerService.Host);
            Internal.ServerManager.SetSetting("Port", GeneralServerService.Port);

            GeneralServerService.ServerInstance.Start();

            Main Browser = new();
            Browser.Show();
        }

        private void GlobalUnhandledExceptionHandler(object sender, UnhandledExceptionEventArgs e)
        {
            Exception ex = (Exception)e.ExceptionObject;
            System.Windows.MessageBox.Show(ex.Message + "\n" + ex.InnerException + "\n" + ex.StackTrace + "\n" + ex.Data + "\n" + ex.TargetSite + "\n" + ex.HResult + "\n" + ex.Source + "\n" + ex.HelpLink);
            //var logger = NLog.LogManager.GetCurrentClassLogger();
            //logger.Error($"UNHANDELED EXCEPTION START");
            //logger.Error($"Application crashed: {ex.Message}.");
            //logger.Error($"Inner exception: {ex.InnerException}.");
            //logger.Error($"Stack trace: {ex.StackTrace}.");
            //logger.Error($"UNHANDELED EXCEPTION FINISH");
            Close();
        }
    }
}