using CefSharp;
using CefSharp.Wpf;
using System.Globalization;
using System.IO;
using System.Windows;
using Application = System.Windows.Application;
using SHC = Skylark.Helper.Culture;
using SMC = Sucrose.Memory.Constant;
using SMMI = Sucrose.Manager.Manage.Internal;
using SMR = Sucrose.Memory.Readonly;
using SRHR = Sucrose.Resources.Helper.Resources;
using SSMI = Sucrose.Signal.Manage.Internal;
using SSSHI = Sucrose.Shared.Space.Helper.Instance;
using SSSHW = Sucrose.Shared.Space.Helper.Watchdog;
using SSSSWSS = Sucrose.Shared.Signal.Services.WebsiterSignalService;
using SSWW = Sucrose.Shared.Watchdog.Watch;

namespace Sucrose.WPF.CS
{
    /// <summary>
    /// Interaction logic for App.xaml
    /// </summary>
    public partial class App : Application
    {
        private static string Culture => SMMI.GeneralSettingManager.GetSetting(SMC.CultureName, SHC.CurrentUITwoLetterISOLanguageName);

        private static bool HasError { get; set; } = true;

        public App()
        {
            System.Windows.Forms.Application.SetUnhandledExceptionMode(UnhandledExceptionMode.Automatic);

            System.Windows.Forms.Application.ThreadException += async (s, e) =>
            {
                Exception Exception = e.Exception;

                await SSWW.Watch_ThreadException(Exception);

                //Close();
                Message(Exception.Message);
            };

            AppDomain.CurrentDomain.FirstChanceException += async (s, e) =>
            {
                Exception Exception = e.Exception;

                await SSWW.Watch_FirstChanceException(Exception);

                //Close();
                //Message(Exception.Message);
            };

            AppDomain.CurrentDomain.UnhandledException += async (s, e) =>
            {
                Exception Exception = (Exception)e.ExceptionObject;

                await SSWW.Watch_GlobalUnhandledException(Exception);

                //Close();
                Message(Exception.Message);
            };

            TaskScheduler.UnobservedTaskException += async (s, e) =>
            {
                Exception Exception = e.Exception;

                await SSWW.Watch_UnobservedTaskException(Exception);

                e.SetObserved();

                //Close();
                Message(Exception.Message);
            };

            Current.DispatcherUnhandledException += async (s, e) =>
            {
                Exception Exception = e.Exception;

                await SSWW.Watch_DispatcherUnhandledException(Exception);

                e.Handled = true;

                //Close();
                Message(Exception.Message);
            };

            SHC.All = new CultureInfo(Culture, true);

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
            if (HasError)
            {
                HasError = false;

                string Path = SMMI.CefSharpLiveLogManager.LogFile();

                SSSHW.Start(Message, Path);

                Close();
            }
        }

        protected void Configure()
        {
            SSMI.WebsiterManager.StartChannel(SSSSWSS.Handler);

            Main Browser = new();
            Browser.Show();
        }

        protected override void OnExit(ExitEventArgs e)
        {
            base.OnExit(e);

            SSMI.WebsiterManager.StopChannel();

            Close();
        }

        protected override void OnStartup(StartupEventArgs e)
        {
            base.OnStartup(e);

            SRHR.SetLanguage(Culture);

            ShutdownMode = ShutdownMode.OnExplicitShutdown;

            if (SSSHI.Basic(SMR.LiveMutex, "Sucrose.WPF.CS.exe"))
            {
                Configure();
            }
            else
            {
                Close();
            }
        }
    }
}