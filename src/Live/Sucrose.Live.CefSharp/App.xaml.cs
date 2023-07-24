using CefSharp;
using CefSharp.Wpf;
using System.Globalization;
using System.IO;
using System.Windows;
using Application = System.Windows.Application;
using SEWTT = Skylark.Enum.WindowsThemeType;
using SGMR = Sucrose.Globalization.Manage.Resources;
using SMC = Sucrose.Memory.Constant;
using SMMI = Sucrose.Manager.Manage.Internal;
using SMR = Sucrose.Memory.Readonly;
using SSDEWT = Sucrose.Shared.Dependency.Enum.WallpaperType;
using SSECSVV = Sucrose.Shared.Engine.CefSharp.View.Video;
using SSECSVW = Sucrose.Shared.Engine.CefSharp.View.Web;
using SSECSVYT = Sucrose.Shared.Engine.CefSharp.View.YouTube;
using SSEHR = Sucrose.Shared.Engine.Helper.Run;
using SSEMI = Sucrose.Shared.Engine.Manage.Internal;
using SSTHI = Sucrose.Shared.Theme.Helper.Info;
using SSTHP = Sucrose.Shared.Theme.Helper.Properties;
using SSTHC = Sucrose.Shared.Theme.Helper.Compatible;
using SSTHV = Sucrose.Shared.Theme.Helper.Various;
using SSWDEMB = Sucrose.Shared.Watchdog.DarkErrorMessageBox;
using SSWLEMB = Sucrose.Shared.Watchdog.LightErrorMessageBox;
using SSWW = Sucrose.Shared.Watchdog.Watch;
using SWHWT = Skylark.Wing.Helper.WindowsTheme;

namespace Sucrose.Live.CefSharp
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

                string Path = SMMI.CefSharpLiveLogManager.LogFile();

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
                string CompatiblePath = Path.Combine(Directory, Folder, SMR.SucroseCompatible);

                if (File.Exists(InfoPath))
                {
#if NET48_OR_GREATER && DEBUG
                    CefRuntime.SubscribeAnyCpuAssemblyResolver();
#endif

                    CefSettings Settings = new()
                    {
                        CachePath = Path.Combine(SMR.AppDataPath, SMR.AppName, SMR.CacheFolder, SMR.CefSharp)
                    };

                    SSEMI.BrowserSettings.CefSharp = SMMI.EngineSettingManager.GetSetting(SMC.CefArguments, new Dictionary<string, string>());

                    if (!SSEMI.BrowserSettings.CefSharp.Any())
                    {
                        SSEMI.BrowserSettings.CefSharp = SSEMI.CefArguments;

                        SMMI.EngineSettingManager.SetSetting(SMC.CefArguments, SSEMI.BrowserSettings.CefSharp);
                    }

                    foreach (KeyValuePair<string, string> Argument in SSEMI.BrowserSettings.CefSharp)
                    {
                        Settings.CefCommandLineArgs.Add(Argument.Key, Argument.Value);
                    }

                    //Example of checking if a call to Cef.Initialize has already been made, we require this for
                    //our .Net 5.0 Single File Publish example, you don't typically need to perform this check
                    //if you call Cef.Initialze within your WPF App constructor.
                    if (!Cef.IsInitialized)
                    {
                        //Perform dependency check to make sure all relevant resources are in our output directory.
                        Cef.Initialize(Settings, performDependencyCheck: true, browserProcessHandler: null);
                    }

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
                            SSEMI.Properties = SSTHP.ReadJson(PropertiesPath);
                            SSEMI.Properties.State = true;
                        }

                        if (File.Exists(CompatiblePath))
                        {
                            SSEMI.Compatible = SSTHC.ReadJson(CompatiblePath);
                            SSEMI.Compatible.State = true;
                        }

                        switch (Info.Type)
                        {
                            case SSDEWT.Web:
                                SSECSVW Web = new(Source);
                                Web.Show();
                                break;
                            case SSDEWT.Video:
                                SSECSVV Video = new(Source);
                                Video.Show();
                                break;
                            case SSDEWT.YouTube:
                                SSECSVYT YouTube = new(Source);
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

            Cef.Shutdown();

            Close();
        }

        protected override void OnStartup(StartupEventArgs e)
        {
            base.OnStartup(e);

            ShutdownMode = ShutdownMode.OnExplicitShutdown;

            if (Mutex.WaitOne(TimeSpan.Zero, true) && SSEHR.Check())
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