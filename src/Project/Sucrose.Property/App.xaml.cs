using HandyControl.Tools;
using System.Globalization;
using System.IO;
using System.Windows;
using Application = System.Windows.Application;
using SHC = Skylark.Helper.Culture;
using SMMI = Sucrose.Manager.Manage.Internal;
using SMMM = Sucrose.Manager.Manage.Manager;
using SMR = Sucrose.Memory.Readonly;
using SPMI = Sucrose.Property.Manage.Internal;
using SPVMW = Sucrose.Property.View.MainWindow;
using SRHR = Sucrose.Resources.Helper.Resources;
using SSDEWT = Sucrose.Shared.Dependency.Enum.WallpaperType;
using SSSHI = Sucrose.Shared.Space.Helper.Instance;
using SSSHW = Sucrose.Shared.Space.Helper.Watchdog;
using SSTHI = Sucrose.Shared.Theme.Helper.Info;
using SSTHP = Sucrose.Shared.Theme.Helper.Properties;
using SSWW = Sucrose.Shared.Watchdog.Watch;

namespace Sucrose.Property
{
    /// <summary>
    /// Interaction logic for App.xaml
    /// </summary>
    public partial class App : Application
    {
        private static bool HasError { get; set; } = true;

        public App()
        {
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

            ConfigHelper.Instance.SetLang(SMMM.Culture);
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

                string Path = SMMI.PropertyLogManager.LogFile();

                SSSHW.Start(Message, Path);

                Close();
            }
        }

        protected void Configure()
        {
            if (SMMI.LibrarySettingManager.CheckFile() && !string.IsNullOrEmpty(SMMM.LibrarySelected))
            {
                string PropertiesPath = Path.Combine(SMMM.LibraryLocation, SMMM.LibrarySelected, SMR.SucroseProperties);

                if (File.Exists(PropertiesPath))
                {
                    string InfoPath = Path.Combine(SMMM.LibraryLocation, SMMM.LibrarySelected, SMR.SucroseInfo);

                    if (File.Exists(InfoPath))
                    {
                        SSTHI Info = SSTHI.ReadJson(InfoPath);

                        if (Info.Type == SSDEWT.Web)
                        {
                            string PropertiesCache = Path.Combine(SMR.AppDataPath, SMR.AppName, SMR.CacheFolder, SMR.Properties);
                            string PropertiesFile = Path.Combine(PropertiesCache, $"{SMMM.LibrarySelected}.json");

                            if (!Directory.Exists(PropertiesCache))
                            {
                                Directory.CreateDirectory(PropertiesCache);
                            }

                            if (!File.Exists(PropertiesFile))
                            {
                                File.Copy(PropertiesPath, PropertiesFile, true);
                            }

                            SPMI.Properties = SSTHP.ReadJson(PropertiesFile);

                            SPVMW MainWindow = new();
                            MainWindow.ShowDialog();
                        }
                    }
                }
            }

            Close();
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

            SRHR.SetLanguage(SMMM.Culture);

            ShutdownMode = ShutdownMode.OnExplicitShutdown;

            if (SSSHI.Basic(SMR.PropertyMutex, SMR.Property))
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