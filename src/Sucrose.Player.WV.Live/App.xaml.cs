using Microsoft.Web.WebView2.Core;
using System.IO;
using System.Windows;
using Application = System.Windows.Application;
using SEWTT = Skylark.Enum.WindowsThemeType;
using SGMR = Sucrose.Globalization.Manage.Resources;
using SMC = Sucrose.Memory.Constant;
using SMMI = Sucrose.Manager.Manage.Internal;
using SMR = Sucrose.Memory.Readonly;
using SPWVMI = Sucrose.Player.WV.Manage.Internal;
using SWDEMB = Sucrose.Watchdog.DarkErrorMessageBox;
using SWHWT = Skylark.Wing.Helper.WindowsTheme;
using SWLEMB = Sucrose.Watchdog.LightErrorMessageBox;
using SWW = Sucrose.Watchdog.Watch;

namespace Sucrose.Player.WV.Live
{
    /// <summary>
    /// Interaction logic for App.xaml
    /// </summary>
    public partial class App : Application
    {
        private static string Culture { get; set; } = SMMI.GeneralSettingManager.GetSetting(SMC.CultureName, SGMR.CultureInfo.Name);

        private static SEWTT Theme { get; set; } = SMMI.GeneralSettingManager.GetSetting(SMC.ThemeType, SWHWT.GetTheme());

        private static Mutex Mutex { get; } = new(true, SMR.EngineMutex);

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

                string Path = SMMI.WebViewPlayerLogManager.LogFile();

                switch (Theme)
                {
                    case SEWTT.Dark:
                        SWDEMB DarkMessageBox = new(Culture, Message, Path);
                        DarkMessageBox.ShowDialog();
                        break;
                    default:
                        SWLEMB LightMessageBox = new(Culture, Message, Path);
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

        protected void Configure()
        {
            CoreWebView2EnvironmentOptions Options = new()
            {
                Language = Culture,
                AdditionalBrowserArguments = "--enable-media-stream --enable-accelerated-video-decode --allow-running-insecure-content --use-fake-ui-for-media-stream --enable-speech-input --enable-usermedia-screen-capture --debug-plugin-loading --allow-outdated-plugins --always-authorize-plugins --enable-npapi"
            };

            Task<CoreWebView2Environment> Environment = CoreWebView2Environment.CreateAsync(null, Path.Combine(SMR.AppDataPath, SMR.AppName, SMR.CacheFolder, SMR.WebView2), Options);

            SPWVMI.EdgePlayer.EnsureCoreWebView2Async(Environment.Result);

            SMMI.EngineSettingManager.SetSetting(SMC.App, SMR.WebViewLive);

            WebView Player = new();
            Player.Show();

            HasStart = true;
        }

        protected override void OnExit(ExitEventArgs e)
        {
            base.OnExit(e);

            //

            Close();
        }

        protected override void OnStartup(StartupEventArgs e)
        {
            base.OnStartup(e);

            if (Mutex.WaitOne(TimeSpan.Zero, true))
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