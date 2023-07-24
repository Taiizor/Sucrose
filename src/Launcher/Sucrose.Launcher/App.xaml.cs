﻿using System.Globalization;
using System.Windows;
using Application = System.Windows.Application;
using SELLT = Skylark.Enum.LevelLogType;
using SEWTT = Skylark.Enum.WindowsThemeType;
using SGCL = Sucrose.Grpc.Common.Launcher;
using SGSGSS = Sucrose.Grpc.Services.GeneralServerService;
using SHC = Skylark.Helper.Culture;
using SMC = Sucrose.Memory.Constant;
using SMMI = Sucrose.Manager.Manage.Internal;
using SMR = Sucrose.Memory.Readonly;
using SSDH = Sucrose.Shared.Discord.Hook;
using SSLCI = Sucrose.Shared.Launcher.Command.Interface;
using SSLMI = Sucrose.Shared.Launcher.Manage.Internal;
using SSRHR = Sucrose.Shared.Resources.Helper.Resources;
using SSSHP = Sucrose.Shared.Space.Helper.Processor;
using SSSSLSS = Sucrose.Shared.Server.Services.LauncherServerService;
using SSWDEMB = Sucrose.Shared.Watchdog.DarkErrorMessageBox;
using SSWLEMB = Sucrose.Shared.Watchdog.LightErrorMessageBox;
using SSWW = Sucrose.Shared.Watchdog.Watch;
using SWHWT = Skylark.Wing.Helper.WindowsTheme;

namespace Sucrose.Launcher
{
    /// <summary>
    /// Interaction logic for App.xaml
    /// </summary>
    public partial class App : Application
    {
        private static string Culture => SMMI.GeneralSettingManager.GetSetting(SMC.CultureName, SHC.CurrentUITwoLetterISOLanguageName);

        private static SEWTT Theme => SMMI.GeneralSettingManager.GetSetting(SMC.ThemeType, SWHWT.GetTheme());

        private static Mutex Mutex => new(true, SMR.LauncherMutex);

        private static bool HasError { get; set; } = true;

        private static SSDH Discord { get; set; } = new();

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

            SHC.All = new CultureInfo(Culture, true);
        }

        protected void Close()
        {
            SMMI.LauncherLogManager.Log(SELLT.Info, $"Application has been closed.");

            Environment.Exit(0);
            Current.Shutdown();
            Shutdown();
        }

        protected void Message(string Message)
        {
            if (HasError)
            {
                HasError = false;

                string Path = SMMI.LauncherLogManager.LogFile();

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
            SMMI.LauncherLogManager.Log(SELLT.Info, "Configuration initializing..");

            SSLMI.TrayIconManager.Start();

            SGSGSS.ServerCreate(SGCL.BindService(new SSSSLSS()));

            SMMI.LauncherSettingManager.SetSetting(SMC.Host, SGSGSS.Host);
            SMMI.LauncherSettingManager.SetSetting(SMC.Port, SGSGSS.Port);

            SMMI.LauncherLogManager.Log(SELLT.Info, "Configuration initialized..");

            SMMI.LauncherLogManager.Log(SELLT.Info, "Server initializing..");

            SGSGSS.ServerInstance.Start();

            SMMI.LauncherLogManager.Log(SELLT.Info, "Server initialized..");

            SMMI.LauncherLogManager.Log(SELLT.Info, "Discord hook initializing..");

            Discord.Initialize();

            SMMI.LauncherLogManager.Log(SELLT.Info, "Discord hook initialized..");
        }

        protected override void OnExit(ExitEventArgs e)
        {
            base.OnExit(e);

            Discord.Dispose();

            SGSGSS.ServerInstance.KillAsync().Wait();
            //SGSGSS.ServerInstance.ShutdownAsync().Wait();

            SSLMI.TrayIconManager.Dispose();

            Close();
        }

        protected override void OnStartup(StartupEventArgs e)
        {
            base.OnStartup(e);

            SSRHR.SetLanguage(Culture);

            ShutdownMode = ShutdownMode.OnExplicitShutdown;

            SMMI.LauncherLogManager.Log(SELLT.Info, "Application initializing..");

            if (Mutex.WaitOne(TimeSpan.Zero, true) && SSSHP.WorkCount(SMR.Launcher) <= 1)
            {
                SMMI.LauncherLogManager.Log(SELLT.Info, "Application mutex is being releasing.");

                Mutex.ReleaseMutex();

                SMMI.LauncherLogManager.Log(SELLT.Info, "Application mutex is being released.");

                Configure();

                SMMI.LauncherLogManager.Log(SELLT.Info, "Application initialized..");
            }
            else
            {
                SMMI.LauncherLogManager.Log(SELLT.Warning, "Application could not be initialized!");

                SMMI.LauncherLogManager.Log(SELLT.Info, "Application Interface opening..");

                SSLCI.Command();

                SMMI.LauncherLogManager.Log(SELLT.Info, "Application Interface opened..");

                Close();
            }
        }
    }
}