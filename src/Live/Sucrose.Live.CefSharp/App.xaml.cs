using CefSharp;
using CefSharp.Wpf;
using System.Globalization;
using System.IO;
using System.Windows;
using Application = System.Windows.Application;
using SEWTT = Skylark.Enum.WindowsThemeType;
using SHC = Skylark.Helper.Culture;
using SMC = Sucrose.Memory.Constant;
using SMMI = Sucrose.Manager.Manage.Internal;
using SMMM = Sucrose.Manager.Manage.Manager;
using SMR = Sucrose.Memory.Readonly;
using SSDEWT = Sucrose.Shared.Dependency.Enum.WallpaperType;
using SSECSVU = Sucrose.Shared.Engine.CefSharp.View.Url;
using SSECSVV = Sucrose.Shared.Engine.CefSharp.View.Video;
using SSECSVW = Sucrose.Shared.Engine.CefSharp.View.Web;
using SSECSVYT = Sucrose.Shared.Engine.CefSharp.View.YouTube;
using SSEHR = Sucrose.Shared.Engine.Helper.Run;
using SSEMI = Sucrose.Shared.Engine.Manage.Internal;
using SSEMM = Sucrose.Shared.Engine.Manage.Manager;
using SSLHK = Sucrose.Shared.Live.Helper.Kill;
using SSRHR = Sucrose.Shared.Resources.Helper.Resources;
using SSSHI = Sucrose.Shared.Space.Helper.Instance;
using SSSHS = Sucrose.Shared.Space.Helper.Security;
using SSTHC = Sucrose.Shared.Theme.Helper.Compatible;
using SSTHI = Sucrose.Shared.Theme.Helper.Info;
using SSTHP = Sucrose.Shared.Theme.Helper.Properties;
using SSTHV = Sucrose.Shared.Theme.Helper.Various;
using SSWDEMB = Sucrose.Shared.Watchdog.DarkErrorMessageBox;
using SSWLEMB = Sucrose.Shared.Watchdog.LightErrorMessageBox;
using SSWW = Sucrose.Shared.Watchdog.Watch;

namespace Sucrose.Live.CefSharp
{
    /// <summary>
    /// Interaction logic for App.xaml
    /// </summary>
    public partial class App : Application
    {
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

                SSWW.Watch_GlobalUnhandledException(Exception);

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

            SHC.All = new CultureInfo(SMMM.Culture, true);
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

                switch (SSEMM.ThemeType)
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
            if (SMMI.LibrarySettingManager.CheckFile() && !string.IsNullOrEmpty(SMMM.LibrarySelected))
            {
                string InfoPath = Path.Combine(SMMM.LibraryLocation, SMMM.LibrarySelected, SMR.SucroseInfo);
                string PropertiesPath = Path.Combine(SMMM.LibraryLocation, SMMM.LibrarySelected, SMR.SucroseProperties);
                string CompatiblePath = Path.Combine(SMMM.LibraryLocation, SMMM.LibrarySelected, SMR.SucroseCompatible);

                if (File.Exists(InfoPath))
                {
                    SSLHK.StopSubprocess();

#if NET48_OR_GREATER && DEBUG
                    CefRuntime.SubscribeAnyCpuAssemblyResolver();
#endif

                    CefSettings Settings = new()
                    {
                        UserAgent = SMMM.UserAgent,
                        CachePath = Path.Combine(SMR.AppDataPath, SMR.AppName, SMR.CacheFolder, SMR.CefSharp)
                    };

                    SSEMI.BrowserSettings.CefSharp = SMMM.CefArguments;

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
                        Source = Path.Combine(SMMM.LibraryLocation, SMMM.LibrarySelected, Source);
                    }

                    if (SSTHV.IsUrl(Source) || File.Exists(Source))
                    {
                        SSSHS.Apply();

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
                            case SSDEWT.Url:
                                SSECSVU Url = new(Source);
                                Url.Show();
                                break;
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

            SSRHR.SetLanguage(SMMM.Culture);

            ShutdownMode = ShutdownMode.OnExplicitShutdown;

            if (SSSHI.Basic(SMR.LiveMutex, SMR.CefSharpLive) && SSEHR.Check())
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