using System.Runtime.ExceptionServices;
using System.Windows;
using System.Windows.Threading;
using Application = System.Windows.Application;
using SCMI = Sucrose.Common.Manage.Internal;
using SCSTISS = Sucrose.Common.Services.TrayIconServerService;
using SELLT = Skylark.Enum.LevelLogType;
using SEWTT = Skylark.Enum.WindowsThemeType;
using SGCTI = Sucrose.Grpc.Common.TrayIcon;
using SGMR = Sucrose.Globalization.Manage.Resources;
using SGSGSS = Sucrose.Grpc.Services.GeneralServerService;
using SMBDEMB = Sucrose.MessageBox.DarkErrorMessageBox;
using SMBLEMB = Sucrose.MessageBox.LightErrorMessageBox;
using SMC = Sucrose.Memory.Constant;
using SMR = Sucrose.Memory.Readonly;
using STCI = Sucrose.Tray.Command.Interface;
using SWHWT = Skylark.Wing.Helper.WindowsTheme;

namespace Sucrose.WPF.TI
{
    /// <summary>
    /// Interaction logic for App.xaml
    /// </summary>
    public partial class App : Application
    {
        private static string Culture { get; set; } = SCMI.GeneralSettingManager.GetSetting(SMC.CultureName, SGMR.CultureInfo.Name);

        private static SEWTT Theme { get; set; } = SCMI.GeneralSettingManager.GetSetting(SMC.ThemeType, SWHWT.GetTheme());

        private static bool Visible { get; set; } = SCMI.TrayIconSettingManager.GetSetting(SMC.Visible, true);

        private static Mutex Mutex { get; } = new(true, SMR.TrayIconMutex);

        private static bool HasStart { get; set; } = false;

        public App()
        {
            System.Windows.Forms.Application.SetUnhandledExceptionMode(UnhandledExceptionMode.Automatic);
            AppDomain.CurrentDomain.UnhandledException += App_GlobalUnhandledExceptionHandler;
            System.Windows.Forms.Application.ThreadException += Application_ThreadException;
            TaskScheduler.UnobservedTaskException += TaskScheduler_UnobservedTaskException;
            AppDomain.CurrentDomain.FirstChanceException += App_FirstChanceException;
            Current.DispatcherUnhandledException += App_DispatcherUnhandledException;
        }

        protected void Close()
        {
            SCMI.TrayIconLogManager.Log(SELLT.Error, $"Application has been closed.");

            Environment.Exit(0);
            Current.Shutdown();
            Shutdown();
        }

        protected void Message(string Message)
        {
            if (HasStart)
            {
                HasStart = false;

                string Path = SCMI.TrayIconLogManager.LogFile();

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
            SCMI.TrayIconLogManager.Log(SELLT.Info, "Configuration initializing..");

            SCMI.TrayIconManager.Start(Theme, Culture);

            SGSGSS.ServerCreate(SGCTI.BindService(new SCSTISS()));

            SCMI.TrayIconSettingManager.SetSetting(SMC.Host, SGSGSS.Host);
            SCMI.TrayIconSettingManager.SetSetting(SMC.Port, SGSGSS.Port);

            SCMI.TrayIconLogManager.Log(SELLT.Info, "Configuration initialized..");

            SCMI.TrayIconLogManager.Log(SELLT.Info, "Server initializing..");

            SGSGSS.ServerInstance.Start();

            SCMI.TrayIconLogManager.Log(SELLT.Info, "Server initialized..");

            HasStart = true;
        }

        protected override void OnExit(ExitEventArgs e)
        {
            base.OnExit(e);

            SGSGSS.ServerInstance.KillAsync().Wait();
            //SGSGSS.ServerInstance.ShutdownAsync().Wait();

            SCMI.TrayIconManager.Dispose();

            Close();
        }

        protected override void OnStartup(StartupEventArgs e)
        {
            base.OnStartup(e);

            SCMI.TrayIconLogManager.Log(SELLT.Info, "Application initializing..");

            if (Mutex.WaitOne(TimeSpan.Zero, true) && Visible)
            {
                SCMI.TrayIconLogManager.Log(SELLT.Info, "Application mutex is being releasing.");

                Mutex.ReleaseMutex();

                SCMI.TrayIconLogManager.Log(SELLT.Info, "Application mutex is being released.");

                Configure();

                SCMI.TrayIconLogManager.Log(SELLT.Info, "Application initialized..");
            }
            else
            {
                SCMI.TrayIconLogManager.Log(SELLT.Warning, "Application could not be initialized!");

                SCMI.TrayIconLogManager.Log(SELLT.Info, "Application Interface opening..");

                STCI.Command();

                SCMI.TrayIconLogManager.Log(SELLT.Info, "Application Interface opened..");

                Close();
            }
        }

        private void Application_ThreadException(object sender, ThreadExceptionEventArgs e)
        {
            Exception Exception = e.Exception;

            SCMI.TrayIconLogManager.Log(SELLT.Error, $"THREAD EXCEPTION START");
            SCMI.TrayIconLogManager.Log(SELLT.Error, $"Application crashed: {Exception.Message}.");
            SCMI.TrayIconLogManager.Log(SELLT.Error, $"Inner exception: {Exception.InnerException}.");
            SCMI.TrayIconLogManager.Log(SELLT.Error, $"Stack trace: {Exception.StackTrace}.");
            SCMI.TrayIconLogManager.Log(SELLT.Error, $"THREAD EXCEPTION FINISH");

            //Close();
            Message(Exception.Message);
        }

        private void App_FirstChanceException(object sender, FirstChanceExceptionEventArgs e)
        {
            Exception Exception = e.Exception;

            SCMI.TrayIconLogManager.Log(SELLT.Error, $"FIRST CHANCE EXCEPTION START");
            SCMI.TrayIconLogManager.Log(SELLT.Error, $"Application crashed: {Exception.Message}.");
            SCMI.TrayIconLogManager.Log(SELLT.Error, $"Inner exception: {Exception.InnerException}.");
            SCMI.TrayIconLogManager.Log(SELLT.Error, $"Stack trace: {Exception.StackTrace}.");
            SCMI.TrayIconLogManager.Log(SELLT.Error, $"FIRST CHANCE EXCEPTION FINISH");

            //Close();
            //Message(Exception.Message);
        }

        protected void App_GlobalUnhandledExceptionHandler(object sender, UnhandledExceptionEventArgs e)
        {
            Exception Exception = (Exception)e.ExceptionObject;

            SCMI.TrayIconLogManager.Log(SELLT.Error, $"GLOBAL UNHANDLED EXCEPTION START");
            SCMI.TrayIconLogManager.Log(SELLT.Error, $"Application crashed: {Exception.Message}.");
            SCMI.TrayIconLogManager.Log(SELLT.Error, $"Inner exception: {Exception.InnerException}.");
            SCMI.TrayIconLogManager.Log(SELLT.Error, $"Stack trace: {Exception.StackTrace}.");
            SCMI.TrayIconLogManager.Log(SELLT.Error, $"GLOBAL UNHANDLED EXCEPTION FINISH");

            //Close();
            Message(Exception.Message);
        }

        private void App_DispatcherUnhandledException(object sender, DispatcherUnhandledExceptionEventArgs e)
        {
            Exception Exception = e.Exception;

            SCMI.TrayIconLogManager.Log(SELLT.Error, $"DISPATCHER UNHANDLED EXCEPTION START");
            SCMI.TrayIconLogManager.Log(SELLT.Error, $"Application crashed: {Exception.Message}.");
            SCMI.TrayIconLogManager.Log(SELLT.Error, $"Inner exception: {Exception.InnerException}.");
            SCMI.TrayIconLogManager.Log(SELLT.Error, $"Stack trace: {Exception.StackTrace}.");
            SCMI.TrayIconLogManager.Log(SELLT.Error, $"DISPATCHER UNHANDLED EXCEPTION FINISH");

            e.Handled = true;

            //Close();
            Message(Exception.Message);
        }

        private void TaskScheduler_UnobservedTaskException(object sender, UnobservedTaskExceptionEventArgs e)
        {
            Exception Exception = e.Exception;

            SCMI.TrayIconLogManager.Log(SELLT.Error, $"UNOBSERVED TASK EXCEPTION START");
            SCMI.TrayIconLogManager.Log(SELLT.Error, $"Application crashed: {Exception.Message}.");
            SCMI.TrayIconLogManager.Log(SELLT.Error, $"Inner exception: {Exception.InnerException}.");
            SCMI.TrayIconLogManager.Log(SELLT.Error, $"Stack trace: {Exception.StackTrace}.");
            SCMI.TrayIconLogManager.Log(SELLT.Error, $"UNOBSERVED TASK EXCEPTION FINISH");

            e.SetObserved();

            //Close();
            Message(Exception.Message);
        }
    }
}