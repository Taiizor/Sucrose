using Skylark.Enum;
using Skylark.Wing.Helper;
using Sucrose.Common.Manage;
using Sucrose.Common.Services;
using Sucrose.Grpc.Common;
using Sucrose.Grpc.Services;
using Sucrose.Memory;
using Sucrose.Tray.Command;
using System.Diagnostics;
using System.Windows;
using Application = System.Windows.Application;
using SGMR = Sucrose.Globalization.Manage.Resources;

namespace Sucrose.WPF.TI
{
    /// <summary>
    /// Interaction logic for App.xaml
    /// </summary>
    public partial class App : Application
    {
        private static readonly Mutex Mutex = new(true, Readonly.TrayIconMutex);

        public App()
        {
            AppDomain.CurrentDomain.UnhandledException += GlobalUnhandledExceptionHandler;

            Internal.TrayIconLogManager.Log(Manager.LogLevelType.Info, "TrayIcon initializing..");
        }

        protected void Close()
        {
            Environment.Exit(0);
            Current.Shutdown();
            Shutdown();
        }

        protected void Configure()
        {
            WindowsThemeType Theme = Internal.TrayIconSettingManager.GetSetting("ThemeType", WindowsTheme.GetTheme());
            string Culture = Internal.TrayIconSettingManager.GetSetting("CultureName", SGMR.CultureInfo.Name);
            WindowsThemeType Theme = Internal.TrayIconSettingManager.GetSetting("ThemeType", WindowsTheme.GetTheme());

            Internal.TrayIconManager.Start(Theme, Culture);

            GeneralServerService.ServerCreate(TrayIcon.BindService(new TrayIconServerService()));

            Internal.TrayIconSettingManager.SetSetting("ThemeType", Theme);
            Internal.TrayIconSettingManager.SetSetting("CultureName", Culture);
            Internal.TrayIconSettingManager.SetSetting("Host", GeneralServerService.Host);
            Internal.TrayIconSettingManager.SetSetting("Port", GeneralServerService.Port);

            GeneralServerService.ServerInstance.Start();

            Internal.TrayIconLogManager.Log(Manager.LogLevelType.Info, "TrayIcon initialized..");
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

            if (Mutex.WaitOne(TimeSpan.Zero, true) && Internal.TrayIconSettingManager.GetSetting("Visible", true))
            {
                Mutex.ReleaseMutex();
                Configure();
            }
            else
            {
                Interface.Command();
                Close();
            }
        }

        protected void GlobalUnhandledExceptionHandler(object sender, UnhandledExceptionEventArgs e)
        {
            Exception ex = (Exception)e.ExceptionObject;
            System.Windows.MessageBox.Show(ex.Message + "\n" + ex.InnerException + "\n" + ex.StackTrace + "\n" + ex.Data + "\n" + ex.TargetSite + "\n" + ex.HResult + "\n" + ex.Source + "\n" + ex.HelpLink);
            //var logger = NLog.LogManager.GetCurrentClassLogger();
            //logger.Error($"UNHANDELED EXCEPTION START");
            //logger.Error($"Application crashed: {ex.Message}.");
            //logger.Error($"Inner exception: {ex.InnerException}.");
            //logger.Error($"Stack trace: {ex.StackTrace}.");
            //logger.Error($"UNHANDELED EXCEPTION FINISH");
            Close();
        }
    }
}