using CefSharp;
using CefSharp.Wpf;
using Grpc.Core;
using System.IO;
using System.Windows;
using Application = System.Windows.Application;
using SMMI = Sucrose.Manager.Manage.Internal;
using SCSWSS = Sucrose.Common.Services.WebsiterServerService;
using SEWTT = Skylark.Enum.WindowsThemeType;
using SGCW = Sucrose.Grpc.Common.Websiter;
using SGMR = Sucrose.Globalization.Manage.Resources;
using SGSGSS = Sucrose.Grpc.Services.GeneralServerService;
using SMBDEMB = Sucrose.MessageBox.DarkErrorMessageBox;
using SMBLEMB = Sucrose.MessageBox.LightErrorMessageBox;
using SMC = Sucrose.Memory.Constant;
using SMR = Sucrose.Memory.Readonly;
using SWHWT = Skylark.Wing.Helper.WindowsTheme;
using SWW = Sucrose.Watchdog.Watch;

namespace Sucrose.WPF.CS
{
    /// <summary>
    /// Interaction logic for App.xaml
    /// </summary>
    public partial class App : Application
    {
        private static string Culture { get; set; } = SMMI.GeneralSettingManager.GetSetting(SMC.CultureName, SGMR.CultureInfo.Name);

        private static SEWTT Theme { get; set; } = SMMI.GeneralSettingManager.GetSetting(SMC.ThemeType, SWHWT.GetTheme());

        private static Mutex Mutex { get; } = new(true, SMR.CefSharpMutex);

        private static bool HasStart { get; set; } = false;

        public App()
        {
            System.Windows.Forms.Application.SetUnhandledExceptionMode(UnhandledExceptionMode.Automatic);

            System.Windows.Forms.Application.ThreadException += (s, e) =>
            {
                Exception Exception = e.Exception;

                SWW.Watch_ThreadException(Exception);

                //Close();
                Message(Exception.Message);
            };

            AppDomain.CurrentDomain.FirstChanceException += (s, e) =>
            {
                Exception Exception = e.Exception;

                SWW.Watch_FirstChanceException(Exception);

                //Close();
                //Message(Exception.Message);
            };

            AppDomain.CurrentDomain.UnhandledException += (s, e) =>
            {
                Exception Exception = (Exception)e.ExceptionObject;

                SWW.Watch_GlobalUnhandledExceptionHandler(Exception);

                //Close();
                Message(Exception.Message);
            };

            TaskScheduler.UnobservedTaskException += (s, e) =>
            {
                Exception Exception = e.Exception;

                SWW.Watch_UnobservedTaskException(Exception);

                e.SetObserved();

                //Close();
                Message(Exception.Message);
            };

            Current.DispatcherUnhandledException += (s, e) =>
            {
                Exception Exception = e.Exception;

                SWW.Watch_DispatcherUnhandledException(Exception);

                e.Handled = true;

                //Close();
                Message(Exception.Message);
            };

#if NET48_OR_GREATER && DEBUG
            CefRuntime.SubscribeAnyCpuAssemblyResolver();
#endif

            CefSettings Settings = new()
            {
                CachePath = Path.Combine(SMR.AppDataPath, SMR.AppName, SMR.CacheFolder, SMR.CefSharp)
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

                string Path = SMMI.CefSharpLogManager.LogFile();

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

                SMMI.ServerManager.SetSetting(SMC.Host, SGSGSS.Host);
                SMMI.ServerManager.SetSetting(SMC.Port, SGSGSS.Port);

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
    }
}