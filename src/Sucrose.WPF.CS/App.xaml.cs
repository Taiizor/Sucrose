using CefSharp;
using CefSharp.Wpf;
using Grpc.Core;
using System.IO;
using System.Runtime.ExceptionServices;
using System.Windows;
using System.Windows.Threading;
using Application = System.Windows.Application;
using SCMI = Sucrose.Common.Manage.Internal;
using SCSWSS = Sucrose.Common.Services.WebsiterServerService;
using SELLT = Skylark.Enum.LevelLogType;
using SEWTT = Skylark.Enum.WindowsThemeType;
using SGCW = Sucrose.Grpc.Common.Websiter;
using SGMR = Sucrose.Globalization.Manage.Resources;
using SGSGSS = Sucrose.Grpc.Services.GeneralServerService;
using SMBDEMB = Sucrose.MessageBox.DarkErrorMessageBox;
using SMBLEMB = Sucrose.MessageBox.LightErrorMessageBox;
using SMC = Sucrose.Memory.Constant;
using SMR = Sucrose.Memory.Readonly;
using SWHWT = Skylark.Wing.Helper.WindowsTheme;

namespace Sucrose.WPF.CS
{
    /// <summary>
    /// Interaction logic for App.xaml
    /// </summary>
    public partial class App : Application
    {
        private static string Culture { get; set; } = SCMI.GeneralSettingManager.GetSetting(SMC.CultureName, SGMR.CultureInfo.Name);

        private static SEWTT Theme { get; set; } = SCMI.GeneralSettingManager.GetSetting(SMC.ThemeType, SWHWT.GetTheme());

        private static Mutex Mutex { get; } = new(true, SMR.CefSharpMutex);

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
                CachePath = Path.Combine(SMR.AppDataPath, SMR.AppName, SMR.CefSharp, SMR.CacheFolder)
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

                string Path = SCMI.CefSharpLogManager.LogFile();

                switch (Theme)
                {
                    case SEWTT.Dark:
                        SMBDEMB DarkMessageBox = new(Culture, Message, Path);
                        DarkMessageBox.ShowDialog();
                        break;
                    default:
                        SMBLEMB LightMessageBox = new(Culture, Message, Path);
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

            SGSGSS.ServerInstance.KillAsync().Wait();
            //SGSGSS.ServerInstance.ShutdownAsync().Wait();

            Close();
        }

        protected override void OnStartup(StartupEventArgs e)
        {
            base.OnStartup(e);

            if (Mutex.WaitOne(TimeSpan.Zero, true))
            {
                SGSGSS.ServerCreate(new List<ServerServiceDefinition>
                {
                    SGCW.BindService(new SCSWSS())
                });

                SCMI.ServerManager.SetSetting(SMC.Host, SGSGSS.Host);
                SCMI.ServerManager.SetSetting(SMC.Port, SGSGSS.Port);

                SGSGSS.ServerInstance.Start();

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

            SCMI.CefSharpLogManager.Log(SELLT.Error, $"THREAD EXCEPTION START");
            SCMI.CefSharpLogManager.Log(SELLT.Error, $"Application crashed: {Exception.Message}.");
            SCMI.CefSharpLogManager.Log(SELLT.Error, $"Inner exception: {Exception.InnerException}.");
            SCMI.CefSharpLogManager.Log(SELLT.Error, $"Stack trace: {Exception.StackTrace}.");
            SCMI.CefSharpLogManager.Log(SELLT.Error, $"THREAD EXCEPTION FINISH");

            //Close();
            Message(Exception.Message);
        }

        private void App_FirstChanceException(object sender, FirstChanceExceptionEventArgs e)
        {
            Exception Exception = e.Exception;

            SCMI.CefSharpLogManager.Log(SELLT.Error, $"FIRST CHANCE EXCEPTION START");
            SCMI.CefSharpLogManager.Log(SELLT.Error, $"Application crashed: {Exception.Message}.");
            SCMI.CefSharpLogManager.Log(SELLT.Error, $"Inner exception: {Exception.InnerException}.");
            SCMI.CefSharpLogManager.Log(SELLT.Error, $"Stack trace: {Exception.StackTrace}.");
            SCMI.CefSharpLogManager.Log(SELLT.Error, $"FIRST CHANCE EXCEPTION FINISH");

            //Close();
            //Message(Exception.Message);
        }

        protected void App_GlobalUnhandledExceptionHandler(object sender, UnhandledExceptionEventArgs e)
        {
            Exception Exception = (Exception)e.ExceptionObject;

            SCMI.CefSharpLogManager.Log(SELLT.Error, $"GLOBAL UNHANDLED EXCEPTION START");
            SCMI.CefSharpLogManager.Log(SELLT.Error, $"Application crashed: {Exception.Message}.");
            SCMI.CefSharpLogManager.Log(SELLT.Error, $"Inner exception: {Exception.InnerException}.");
            SCMI.CefSharpLogManager.Log(SELLT.Error, $"Stack trace: {Exception.StackTrace}.");
            SCMI.CefSharpLogManager.Log(SELLT.Error, $"GLOBAL UNHANDLED EXCEPTION FINISH");

            //Close();
            Message(Exception.Message);
        }

        private void App_DispatcherUnhandledException(object sender, DispatcherUnhandledExceptionEventArgs e)
        {
            Exception Exception = e.Exception;

            SCMI.CefSharpLogManager.Log(SELLT.Error, $"DISPATCHER UNHANDLED EXCEPTION START");
            SCMI.CefSharpLogManager.Log(SELLT.Error, $"Application crashed: {Exception.Message}.");
            SCMI.CefSharpLogManager.Log(SELLT.Error, $"Inner exception: {Exception.InnerException}.");
            SCMI.CefSharpLogManager.Log(SELLT.Error, $"Stack trace: {Exception.StackTrace}.");
            SCMI.CefSharpLogManager.Log(SELLT.Error, $"DISPATCHER UNHANDLED EXCEPTION FINISH");

            e.Handled = true;

            //Close();
            Message(Exception.Message);
        }

        private void TaskScheduler_UnobservedTaskException(object sender, UnobservedTaskExceptionEventArgs e)
        {
            Exception Exception = e.Exception;

            SCMI.CefSharpLogManager.Log(SELLT.Error, $"UNOBSERVED TASK EXCEPTION START");
            SCMI.CefSharpLogManager.Log(SELLT.Error, $"Application crashed: {Exception.Message}.");
            SCMI.CefSharpLogManager.Log(SELLT.Error, $"Inner exception: {Exception.InnerException}.");
            SCMI.CefSharpLogManager.Log(SELLT.Error, $"Stack trace: {Exception.StackTrace}.");
            SCMI.CefSharpLogManager.Log(SELLT.Error, $"UNOBSERVED TASK EXCEPTION FINISH");

            e.SetObserved();

            //Close();
            Message(Exception.Message);
        }
    }
}