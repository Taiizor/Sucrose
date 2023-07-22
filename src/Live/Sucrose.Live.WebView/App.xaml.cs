using Microsoft.Web.WebView2.Core;
using System.Globalization;
using System.IO;
using System.Windows;
using Application = System.Windows.Application;
using SESHR = Sucrose.Engine.Shared.Helper.Run;
using SESMI = Sucrose.Engine.Shared.Manage.Internal;
using SEWTT = Skylark.Enum.WindowsThemeType;
using SEWVMI = Sucrose.Engine.WV.Manage.Internal;
using SEWVVV = Sucrose.Engine.WV.View.Video;
using SEWVVW = Sucrose.Engine.WV.View.Web;
using SEWVVYT = Sucrose.Engine.WV.View.YouTube;
using SGMR = Sucrose.Globalization.Manage.Resources;
using SMC = Sucrose.Memory.Constant;
using SMMI = Sucrose.Manager.Manage.Internal;
using SMR = Sucrose.Memory.Readonly;
using SSDEWT = Sucrose.Shared.Dependency.Enum.WallpaperType;
using SSTHI = Sucrose.Shared.Theme.Helper.Info;
using SSTHP = Sucrose.Shared.Theme.Helper.Properties;
using SSTHV = Sucrose.Shared.Theme.Helper.Various;
using SSWDEMB = Sucrose.Shared.Watchdog.DarkErrorMessageBox;
using SSWLEMB = Sucrose.Shared.Watchdog.LightErrorMessageBox;
using SSWW = Sucrose.Shared.Watchdog.Watch;
using SWHWT = Skylark.Wing.Helper.WindowsTheme;

namespace Sucrose.Live.WebView
{
    /// <summary>
    /// Interaction logic for App.xaml
    /// </summary>
    public partial class App : Application
    {
        private static string Directory => SMMI.EngineSettingManager.GetSetting(SMC.Directory, Path.Combine(SMR.DocumentsPath, SMR.AppName));

        private static string Culture => SMMI.GeneralSettingManager.GetSetting(SMC.CultureName, SGMR.CultureInfo.Name);

        private static SEWTT Theme => SMMI.GeneralSettingManager.GetSetting(SMC.ThemeType, SWHWT.GetTheme());

        private static string Folder => SMMI.EngineSettingManager.GetSetting(SMC.Folder, string.Empty);

        private static Mutex Mutex => new(true, SMR.LiveMutex);

        private static bool HasError { get; set; } = true;

        public App()
        {
            System.Windows.Forms.Application.SetUnhandledExceptionMode(UnhandledExceptionMode.Automatic);

            System.Windows.Forms.Application.ThreadException += (s, e) =>
            {
                Exception Exception = e.Exception;

                SSWW.Watch_ThreadException(Exception);

                //Close();
                Message(Exception.Message);
            };

            AppDomain.CurrentDomain.FirstChanceException += (s, e) =>
            {
                Exception Exception = e.Exception;

                SSWW.Watch_FirstChanceException(Exception);

                //Close();
                //Message(Exception.Message);
            };

            AppDomain.CurrentDomain.UnhandledException += (s, e) =>
            {
                Exception Exception = (Exception)e.ExceptionObject;

                SSWW.Watch_GlobalUnhandledExceptionHandler(Exception);

                //Close();
                Message(Exception.Message);
            };

            TaskScheduler.UnobservedTaskException += (s, e) =>
            {
                Exception Exception = e.Exception;

                SSWW.Watch_UnobservedTaskException(Exception);

                e.SetObserved();

                //Close();
                Message(Exception.Message);
            };

            Current.DispatcherUnhandledException += (s, e) =>
            {
                Exception Exception = e.Exception;

                SSWW.Watch_DispatcherUnhandledException(Exception);

                e.Handled = true;

                //Close();
                Message(Exception.Message);
            };

            SGMR.CultureInfo = new CultureInfo(Culture, true);
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

                string Path = SMMI.WebViewLiveLogManager.LogFile();

                switch (Theme)
                {
                    case SEWTT.Dark:
                        SSWDEMB DarkMessageBox = new(Message, Path);
                        DarkMessageBox.ShowDialog();
                        break;
                    default:
                        SSWLEMB LightMessageBox = new(Message, Path);
                        LightMessageBox.ShowDialog();
                        break;
                }

                Close();
            }
        }

        protected void Configure()
        {
            if (SMMI.EngineSettingManager.CheckFile() && !string.IsNullOrEmpty(Folder))
            {
                string InfoPath = Path.Combine(Directory, Folder, SMR.SucroseInfo);
                string PropertiesPath = Path.Combine(Directory, Folder, SMR.SucroseProperties);

                if (File.Exists(InfoPath))
                {
                    CoreWebView2EnvironmentOptions Options = new()
                    {
                        Language = Culture
                    };

                    SESMI.BrowserSettings.WebView = SMMI.EngineSettingManager.GetSetting(SMC.WebArguments, new List<string>());

                    if (!SESMI.BrowserSettings.WebView.Any())
                    {
                        SESMI.BrowserSettings.WebView = SESMI.WebArguments;

                        SMMI.EngineSettingManager.SetSetting(SMC.WebArguments, SESMI.BrowserSettings.WebView);
                    }

                    Options.AdditionalBrowserArguments = string.Join(" ", SESMI.BrowserSettings.WebView);

                    Task<CoreWebView2Environment> Environment = CoreWebView2Environment.CreateAsync(null, Path.Combine(SMR.AppDataPath, SMR.AppName, SMR.CacheFolder, SMR.WebView2), Options);

                    SEWVMI.WebEngine.EnsureCoreWebView2Async(Environment.Result);

                    SSTHI Info = SSTHI.ReadJson(InfoPath);

                    string Source = Info.Source;

                    if (!SSTHV.IsUrl(Source))
                    {
                        Source = Path.Combine(Directory, Folder, Source);
                    }

                    if (SSTHV.IsUrl(Source) || File.Exists(Source))
                    {
                        if (File.Exists(PropertiesPath))
                        {
                            SESMI.Properties = SSTHP.ReadJson(PropertiesPath);
                            SESMI.Properties.State = true;
                        }

                        switch (Info.Type)
                        {
                            case SSDEWT.Web:
                                SEWVVW Web = new(Source);
                                Web.Show();
                                break;
                            case SSDEWT.Video:
                                SEWVVV Video = new(Source);
                                Video.Show();
                                break;
                            case SSDEWT.YouTube:
                                SEWVVYT YouTube = new(Source);
                                YouTube.Show();
                                break;
                            default:
                                Close();
                                break;
                        }
                    }
                    else
                    {
                        Close();
                    }
                }
                else
                {
                    Close();
                }
            }
            else
            {
                Close();
            }
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

            if (Mutex.WaitOne(TimeSpan.Zero, true) && SESHR.Check())
            {
                Mutex.ReleaseMutex();

                Configure();
            }
            else
            {
                Close();
            }
        }
    }
}