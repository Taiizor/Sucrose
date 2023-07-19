using CefSharp;
using CefSharp.Wpf;
using System.Globalization;
using System.IO;
using System.Windows;
using Application = System.Windows.Application;
using SDEWT = Sucrose.Dependency.Enum.WallpaperType;
using SECSVV = Sucrose.Engine.CS.View.Video;
using SECSVW = Sucrose.Engine.CS.View.Web;
using SECSVYT = Sucrose.Engine.CS.View.YouTube;
using SESHR = Sucrose.Engine.Shared.Helper.Run;
using SESMI = Sucrose.Engine.Shared.Manage.Internal;
using SEWTT = Skylark.Enum.WindowsThemeType;
using SGMR = Sucrose.Globalization.Manage.Resources;
using SMC = Sucrose.Memory.Constant;
using SMMI = Sucrose.Manager.Manage.Internal;
using SMR = Sucrose.Memory.Readonly;
using STSHI = Sucrose.Theme.Shared.Helper.Info;
using STSHP = Sucrose.Theme.Shared.Helper.Properties;
using STSHV = Sucrose.Theme.Shared.Helper.Various;
using SWDEMB = Sucrose.Watchdog.DarkErrorMessageBox;
using SWHWT = Skylark.Wing.Helper.WindowsTheme;
using SWLEMB = Sucrose.Watchdog.LightErrorMessageBox;
using SWW = Sucrose.Watchdog.Watch;

namespace Sucrose.Live.CS
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
                        SWDEMB DarkMessageBox = new(Message, Path);
                        DarkMessageBox.ShowDialog();
                        break;
                    default:
                        SWLEMB LightMessageBox = new(Message, Path);
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
#if NET48_OR_GREATER && DEBUG
                    CefRuntime.SubscribeAnyCpuAssemblyResolver();
#endif

                    CefSettings Settings = new()
                    {
                        CachePath = Path.Combine(SMR.AppDataPath, SMR.AppName, SMR.CacheFolder, SMR.CefSharp)
                    };

                    SESMI.BrowserSettings.CefSharp = SMMI.EngineSettingManager.GetSetting(SMC.CefArguments, new Dictionary<string, string>());

                    if (!SESMI.BrowserSettings.CefSharp.Any())
                    {
                        SESMI.BrowserSettings.CefSharp = SESMI.CefArguments;

                        SMMI.EngineSettingManager.SetSetting(SMC.CefArguments, SESMI.BrowserSettings.CefSharp);
                    }

                    foreach (KeyValuePair<string, string> Argument in SESMI.BrowserSettings.CefSharp)
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

                    STSHI Info = STSHI.ReadJson(InfoPath);

                    string Source = Info.Source;

                    if (!STSHV.IsUrl(Source))
                    {
                        Source = Path.Combine(Directory, Folder, Source);
                    }

                    if (STSHV.IsUrl(Source) || File.Exists(Source))
                    {
                        if (File.Exists(PropertiesPath))
                        {
                            SESMI.Properties = STSHP.ReadJson(PropertiesPath);
                        }

                        switch (Info.Type)
                        {
                            case SDEWT.Web:
                                SECSVW Web = new(Source);
                                Web.Show();
                                break;
                            case SDEWT.Video:
                                SECSVV Video = new(Source);
                                Video.Show();
                                break;
                            case SDEWT.YouTube:
                                SECSVYT YouTube = new(Source);
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