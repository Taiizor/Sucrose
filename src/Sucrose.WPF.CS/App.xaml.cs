using CefSharp;
using CefSharp.Wpf;
using Grpc.Core;
using Skylark.Enum;
using Skylark.Wing.Helper;
using Sucrose.Common.Manage;
using Sucrose.Common.Services;
using Sucrose.Grpc.Common;
using Sucrose.Grpc.Services;
using Sucrose.MessageBox;
using Sucrose.Memory;
using System.IO;
using System.Runtime.ExceptionServices;
using System.Windows;
using System.Windows.Threading;
using Application = System.Windows.Application;
using SGMR = Sucrose.Globalization.Manage.Resources;

namespace Sucrose.WPF.CS
{
    /// <summary>
    /// Interaction logic for App.xaml
    /// </summary>
    public partial class App : Application
    {
        private static WindowsThemeType Theme { get; set; } = Internal.GeneralSettingManager.GetSetting(Constant.ThemeType, WindowsTheme.GetTheme());

        private static string Culture { get; set; } = Internal.GeneralSettingManager.GetSetting(Constant.CultureName, SGMR.CultureInfo.Name);

        private static Mutex Mutex { get; } = new(true, Readonly.CefSharpMutex);

        private static bool HasStart { get; set; } = false;

        public App()
        {
            System.Windows.Forms.Application.SetUnhandledExceptionMode(UnhandledExceptionMode.Automatic);
            AppDomain.CurrentDomain.UnhandledException += App_GlobalUnhandledExceptionHandler;
            System.Windows.Forms.Application.ThreadException += Application_ThreadException;
            TaskScheduler.UnobservedTaskException += TaskScheduler_UnobservedTaskException;
            AppDomain.CurrentDomain.FirstChanceException += App_FirstChanceException;
            Current.DispatcherUnhandledException += App_DispatcherUnhandledException;

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
        protected void Message(string Message)
        {
            if (HasStart)
            {
                HasStart = false;

                Message = "Beklenmeyen bir hata oluştu:" + Environment.NewLine + Message;

                switch (Theme)
                {
                    case WindowsThemeType.Dark:
                        DarkErrorMessageBox DarkMessageBox = new(Message);
                        DarkMessageBox.ShowDialog();
                        break;
                    default:
                        LightErrorMessageBox LightMessageBox = new(Message);
                        LightMessageBox.ShowDialog();
                        break;
                }

                Close();
            }
            else
            {
                Close();
            }
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

            if (Mutex.WaitOne(TimeSpan.Zero, true))
            {

                GeneralServerService.ServerCreate(new List<ServerServiceDefinition>
                {
                    Websiter.BindService(new WebsiterServerService())
                });

                Internal.ServerManager.SetSetting(Constant.Host, GeneralServerService.Host);
                Internal.ServerManager.SetSetting(Constant.Port, GeneralServerService.Port);

                GeneralServerService.ServerInstance.Start();

                Main Browser = new();
                Browser.Show();

                HasStart = true;
            }
            else
            {
                Close();
            }
        }

        private void Application_ThreadException(object sender, ThreadExceptionEventArgs e)
        {
            Exception Exception = e.Exception;

            Internal.CefSharpLogManager.Log(LevelLogType.Error, $"THREAD EXCEPTION START");
            Internal.CefSharpLogManager.Log(LevelLogType.Error, $"Application crashed: {Exception.Message}.");
            Internal.CefSharpLogManager.Log(LevelLogType.Error, $"Inner exception: {Exception.InnerException}.");
            Internal.CefSharpLogManager.Log(LevelLogType.Error, $"Stack trace: {Exception.StackTrace}.");
            Internal.CefSharpLogManager.Log(LevelLogType.Error, $"THREAD EXCEPTION FINISH");

            //Close();
            Message(Exception.Message);
        }

        private void App_FirstChanceException(object sender, FirstChanceExceptionEventArgs e)
        {
            Exception Exception = e.Exception;

            Internal.CefSharpLogManager.Log(LevelLogType.Error, $"FIRST CHANCE EXCEPTION START");
            Internal.CefSharpLogManager.Log(LevelLogType.Error, $"Application crashed: {Exception.Message}.");
            Internal.CefSharpLogManager.Log(LevelLogType.Error, $"Inner exception: {Exception.InnerException}.");
            Internal.CefSharpLogManager.Log(LevelLogType.Error, $"Stack trace: {Exception.StackTrace}.");
            Internal.CefSharpLogManager.Log(LevelLogType.Error, $"FIRST CHANCE EXCEPTION FINISH");

            //Close();
            //Message(Exception.Message);
        }

        protected void App_GlobalUnhandledExceptionHandler(object sender, UnhandledExceptionEventArgs e)
        {
            Exception Exception = (Exception)e.ExceptionObject;

            Internal.CefSharpLogManager.Log(LevelLogType.Error, $"GLOBAL UNHANDLED EXCEPTION START");
            Internal.CefSharpLogManager.Log(LevelLogType.Error, $"Application crashed: {Exception.Message}.");
            Internal.CefSharpLogManager.Log(LevelLogType.Error, $"Inner exception: {Exception.InnerException}.");
            Internal.CefSharpLogManager.Log(LevelLogType.Error, $"Stack trace: {Exception.StackTrace}.");
            Internal.CefSharpLogManager.Log(LevelLogType.Error, $"GLOBAL UNHANDLED EXCEPTION FINISH");

            //Close();
            Message(Exception.Message);
        }

        private void App_DispatcherUnhandledException(object sender, DispatcherUnhandledExceptionEventArgs e)
        {
            Exception Exception = e.Exception;

            Internal.CefSharpLogManager.Log(LevelLogType.Error, $"DISPATCHER UNHANDLED EXCEPTION START");
            Internal.CefSharpLogManager.Log(LevelLogType.Error, $"Application crashed: {Exception.Message}.");
            Internal.CefSharpLogManager.Log(LevelLogType.Error, $"Inner exception: {Exception.InnerException}.");
            Internal.CefSharpLogManager.Log(LevelLogType.Error, $"Stack trace: {Exception.StackTrace}.");
            Internal.CefSharpLogManager.Log(LevelLogType.Error, $"DISPATCHER UNHANDLED EXCEPTION FINISH");

            e.Handled = true;

            //Close();
            Message(Exception.Message);
        }

        private void TaskScheduler_UnobservedTaskException(object sender, UnobservedTaskExceptionEventArgs e)
        {
            Exception Exception = e.Exception;

            Internal.CefSharpLogManager.Log(LevelLogType.Error, $"UNOBSERVED TASK EXCEPTION START");
            Internal.CefSharpLogManager.Log(LevelLogType.Error, $"Application crashed: {Exception.Message}.");
            Internal.CefSharpLogManager.Log(LevelLogType.Error, $"Inner exception: {Exception.InnerException}.");
            Internal.CefSharpLogManager.Log(LevelLogType.Error, $"Stack trace: {Exception.StackTrace}.");
            Internal.CefSharpLogManager.Log(LevelLogType.Error, $"UNOBSERVED TASK EXCEPTION FINISH");

            e.SetObserved();

            //Close();
            Message(Exception.Message);
        }
    }
}