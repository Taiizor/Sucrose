using Skylark.Enum;
using Skylark.Wing.Helper;
using Sucrose.Common.Manage;
using Sucrose.Common.Services;
using Sucrose.Grpc.Common;
using Sucrose.Grpc.Services;
using Sucrose.Memory;
using Sucrose.MessageBox;
using Sucrose.Tray.Command;
using System.Runtime.ExceptionServices;
using System.Windows;
using System.Windows.Threading;
using Application = System.Windows.Application;
using SGMR = Sucrose.Globalization.Manage.Resources;

namespace Sucrose.WPF.TI
{
    /// <summary>
    /// Interaction logic for App.xaml
    /// </summary>
    public partial class App : Application
    {
        private static WindowsThemeType Theme { get; set; } = Internal.GeneralSettingManager.GetSetting(Constant.ThemeType, WindowsTheme.GetTheme());

        private static string Culture { get; set; } = Internal.GeneralSettingManager.GetSetting(Constant.CultureName, SGMR.CultureInfo.Name);

        private static bool Visible { get; set; } = Internal.TrayIconSettingManager.GetSetting(Constant.Visible, true);

        private static Mutex Mutex { get; } = new(true, Readonly.TrayIconMutex);

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
            Internal.TrayIconLogManager.Log(LevelLogType.Error, $"Application has been closed.");

            Environment.Exit(0);
            Current.Shutdown();
            Shutdown();
        }

        protected void Message(string Message)
        {
            if (HasStart)
            {
                HasStart = false;

                switch (Theme)
                {
                    case WindowsThemeType.Dark:
                        DarkErrorMessageBox DarkMessageBox = new(Culture, Message);
                        DarkMessageBox.ShowDialog();
                        break;
                    default:
                        LightErrorMessageBox LightMessageBox = new(Culture, Message);
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
            Internal.TrayIconLogManager.Log(LevelLogType.Info, "Configuration initializing..");

            Internal.TrayIconManager.Start(Theme, Culture);

            GeneralServerService.ServerCreate(TrayIcon.BindService(new TrayIconServerService()));

            Internal.TrayIconSettingManager.SetSetting(Constant.Host, GeneralServerService.Host);
            Internal.TrayIconSettingManager.SetSetting(Constant.Port, GeneralServerService.Port);

            Internal.TrayIconLogManager.Log(LevelLogType.Info, "Configuration initialized..");

            Internal.TrayIconLogManager.Log(LevelLogType.Info, "Server initializing..");

            GeneralServerService.ServerInstance.Start();

            Internal.TrayIconLogManager.Log(LevelLogType.Info, "Server initialized..");

            HasStart = true;
        }

        protected override void OnExit(ExitEventArgs e)
        {
            base.OnExit(e);

            GeneralServerService.ServerInstance.KillAsync().Wait();
            //GeneralServerService.ServerInstance.ShutdownAsync().Wait();

            Internal.TrayIconManager.Dispose();

            Close();
        }

        protected override void OnStartup(StartupEventArgs e)
        {
            base.OnStartup(e);

            Internal.TrayIconLogManager.Log(LevelLogType.Info, "Application initializing..");

            if (Mutex.WaitOne(TimeSpan.Zero, true) && Visible)
            {
                Internal.TrayIconLogManager.Log(LevelLogType.Info, "Application mutex is being releasing.");

                Mutex.ReleaseMutex();

                Internal.TrayIconLogManager.Log(LevelLogType.Info, "Application mutex is being released.");

                Configure();

                Internal.TrayIconLogManager.Log(LevelLogType.Info, "Application initialized..");
            }
            else
            {
                Internal.TrayIconLogManager.Log(LevelLogType.Warning, "Application could not be initialized!");

                Internal.TrayIconLogManager.Log(LevelLogType.Info, "Application Interface opening..");

                Interface.Command();

                Internal.TrayIconLogManager.Log(LevelLogType.Info, "Application Interface opened..");

                Close();
            }
        }

        private void Application_ThreadException(object sender, ThreadExceptionEventArgs e)
        {
            Exception Exception = e.Exception;

            Internal.TrayIconLogManager.Log(LevelLogType.Error, $"THREAD EXCEPTION START");
            Internal.TrayIconLogManager.Log(LevelLogType.Error, $"Application crashed: {Exception.Message}.");
            Internal.TrayIconLogManager.Log(LevelLogType.Error, $"Inner exception: {Exception.InnerException}.");
            Internal.TrayIconLogManager.Log(LevelLogType.Error, $"Stack trace: {Exception.StackTrace}.");
            Internal.TrayIconLogManager.Log(LevelLogType.Error, $"THREAD EXCEPTION FINISH");

            //Close();
            Message(Exception.Message);
        }

        private void App_FirstChanceException(object sender, FirstChanceExceptionEventArgs e)
        {
            Exception Exception = e.Exception;

            Internal.TrayIconLogManager.Log(LevelLogType.Error, $"FIRST CHANCE EXCEPTION START");
            Internal.TrayIconLogManager.Log(LevelLogType.Error, $"Application crashed: {Exception.Message}.");
            Internal.TrayIconLogManager.Log(LevelLogType.Error, $"Inner exception: {Exception.InnerException}.");
            Internal.TrayIconLogManager.Log(LevelLogType.Error, $"Stack trace: {Exception.StackTrace}.");
            Internal.TrayIconLogManager.Log(LevelLogType.Error, $"FIRST CHANCE EXCEPTION FINISH");

            //Close();
            //Message(Exception.Message);
        }

        protected void App_GlobalUnhandledExceptionHandler(object sender, UnhandledExceptionEventArgs e)
        {
            Exception Exception = (Exception)e.ExceptionObject;

            Internal.TrayIconLogManager.Log(LevelLogType.Error, $"GLOBAL UNHANDLED EXCEPTION START");
            Internal.TrayIconLogManager.Log(LevelLogType.Error, $"Application crashed: {Exception.Message}.");
            Internal.TrayIconLogManager.Log(LevelLogType.Error, $"Inner exception: {Exception.InnerException}.");
            Internal.TrayIconLogManager.Log(LevelLogType.Error, $"Stack trace: {Exception.StackTrace}.");
            Internal.TrayIconLogManager.Log(LevelLogType.Error, $"GLOBAL UNHANDLED EXCEPTION FINISH");

            //Close();
            Message(Exception.Message);
        }

        private void App_DispatcherUnhandledException(object sender, DispatcherUnhandledExceptionEventArgs e)
        {
            Exception Exception = e.Exception;

            Internal.TrayIconLogManager.Log(LevelLogType.Error, $"DISPATCHER UNHANDLED EXCEPTION START");
            Internal.TrayIconLogManager.Log(LevelLogType.Error, $"Application crashed: {Exception.Message}.");
            Internal.TrayIconLogManager.Log(LevelLogType.Error, $"Inner exception: {Exception.InnerException}.");
            Internal.TrayIconLogManager.Log(LevelLogType.Error, $"Stack trace: {Exception.StackTrace}.");
            Internal.TrayIconLogManager.Log(LevelLogType.Error, $"DISPATCHER UNHANDLED EXCEPTION FINISH");

            e.Handled = true;

            //Close();
            Message(Exception.Message);
        }

        private void TaskScheduler_UnobservedTaskException(object sender, UnobservedTaskExceptionEventArgs e)
        {
            Exception Exception = e.Exception;

            Internal.TrayIconLogManager.Log(LevelLogType.Error, $"UNOBSERVED TASK EXCEPTION START");
            Internal.TrayIconLogManager.Log(LevelLogType.Error, $"Application crashed: {Exception.Message}.");
            Internal.TrayIconLogManager.Log(LevelLogType.Error, $"Inner exception: {Exception.InnerException}.");
            Internal.TrayIconLogManager.Log(LevelLogType.Error, $"Stack trace: {Exception.StackTrace}.");
            Internal.TrayIconLogManager.Log(LevelLogType.Error, $"UNOBSERVED TASK EXCEPTION FINISH");

            e.SetObserved();

            //Close();
            Message(Exception.Message);
        }
    }
}