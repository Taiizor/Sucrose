using System.Globalization;
using System.Windows;
using SEWTT = Skylark.Enum.WindowsThemeType;
using SGMR = Sucrose.Globalization.Manage.Resources;
using SMC = Sucrose.Memory.Constant;
using SMMI = Sucrose.Manager.Manage.Internal;
using SMR = Sucrose.Memory.Readonly;
using SSWDEMB = Sucrose.Shared.Watchdog.DarkErrorMessageBox;
using SWHWT = Skylark.Wing.Helper.WindowsTheme;
using SSWLEMB = Sucrose.Shared.Watchdog.LightErrorMessageBox;
using SSWW = Sucrose.Shared.Watchdog.Watch;

namespace Sucrose.Portal
{
    /// <summary>
    /// Interaction logic for App.xaml
    /// </summary>
    public partial class App : Application
    {
        private static string Culture => SMMI.GeneralSettingManager.GetSetting(SMC.CultureName, SGMR.CultureInfo.Name);

        private static SEWTT Theme => SMMI.GeneralSettingManager.GetSetting(SMC.ThemeType, SWHWT.GetTheme());

        private static Mutex Mutex => new(true, SMR.Portal);

        private static bool HasError { get; set; } = true;

        public App()
        {
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

                string Path = SMMI.PortalLogManager.LogFile();

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

        protected override void OnExit(ExitEventArgs e)
        {
            base.OnExit(e);

            Close();
        }

        protected override void OnStartup(StartupEventArgs e)
        {
            base.OnStartup(e);

            ShutdownMode = ShutdownMode.OnExplicitShutdown;

            if (Mutex.WaitOne(TimeSpan.Zero, true))
            {
                Mutex.ReleaseMutex();

                Main Interface = new();
                Interface.Show();
            }
            else
            {
                Close();
            }
        }
    }
}