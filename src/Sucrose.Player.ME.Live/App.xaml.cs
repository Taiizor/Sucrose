using System.Windows;
using Application = System.Windows.Application;
using SMMI = Sucrose.Manager.Manage.Internal;
using SEWTT = Skylark.Enum.WindowsThemeType;
using SGMR = Sucrose.Globalization.Manage.Resources;
using SMBDEMB = Sucrose.MessageBox.DarkErrorMessageBox;
using SMBLEMB = Sucrose.MessageBox.LightErrorMessageBox;
using SMC = Sucrose.Memory.Constant;
using SMR = Sucrose.Memory.Readonly;
using SWHWT = Skylark.Wing.Helper.WindowsTheme;
using SWW = Sucrose.Watchdog.Watch;

namespace Sucrose.Player.ME.Live
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

                string Path = SMMI.MediaElementLogManager.LogFile();

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

        protected void Configure()
        {
            SMMI.EngineSettingManager.SetSetting(SMC.App, SMR.MediaElementLive);

            MediaElement Player = new();
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