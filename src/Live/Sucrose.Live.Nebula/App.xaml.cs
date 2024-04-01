using System.Globalization;
using System.IO;
using System.Windows;
using Application = System.Windows.Application;
using SHC = Skylark.Helper.Culture;
using SMC = Sucrose.Memory.Constant;
using SMMI = Sucrose.Manager.Manage.Internal;
using SMMM = Sucrose.Manager.Manage.Manager;
using SMR = Sucrose.Memory.Readonly;
using SRHR = Sucrose.Resources.Helper.Resources;
using SSDEWT = Sucrose.Shared.Dependency.Enum.WallpaperType;
using SSEHR = Sucrose.Shared.Engine.Helper.Run;
using SSENVV = Sucrose.Shared.Engine.Nebula.View.Video;
using SSSHI = Sucrose.Shared.Space.Helper.Instance;
using SSSHS = Sucrose.Shared.Space.Helper.Security;
using SSSHW = Sucrose.Shared.Space.Helper.Watchdog;
using SSTHI = Sucrose.Shared.Theme.Helper.Info;
using SSTHV = Sucrose.Shared.Theme.Helper.Various;
using SSWW = Sucrose.Shared.Watchdog.Watch;

namespace Sucrose.Live.Nebula
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

                string Path = SMMI.NebulaLiveLogManager.LogFile();

                SSSHW.Start(Message, Path);

                Close();
            }
        }

        protected void Configure()
        {
            if (SMMI.LibrarySettingManager.CheckFile() && !string.IsNullOrEmpty(SMMM.LibrarySelected))
            {
                string InfoPath = Path.Combine(SMMM.LibraryLocation, SMMM.LibrarySelected, SMR.SucroseInfo);

                if (File.Exists(InfoPath))
                {
                    SSTHI Info = SSTHI.ReadJson(InfoPath);

                    string Source = Info.Source;

                    if (!SSTHV.IsUrl(Source))
                    {
                        Source = Path.Combine(SMMM.LibraryLocation, SMMM.LibrarySelected, Source);
                    }

                    SMMI.BackgroundogSettingManager.SetSetting(SMC.AudioRequired, false);
                    SMMI.BackgroundogSettingManager.SetSetting(SMC.SignalRequired, false);

                    if (SSTHV.IsUrl(Source) || File.Exists(Source))
                    {
                        SSSHS.Apply();

                        switch (Info.Type)
                        {
                            case SSDEWT.Video:
                                SSENVV Video = new(Source);
                                Video.Show();
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

            SRHR.SetLanguage(SMMM.Culture);

            ShutdownMode = ShutdownMode.OnExplicitShutdown;

            if (SSSHI.Basic(SMR.LiveMutex, SMR.NebulaLive) && SSEHR.Check())
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