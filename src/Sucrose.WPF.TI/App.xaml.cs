using System.Windows;
using Sucrose.Common.Manage;
using Sucrose.Common.Services;
using Application = System.Windows.Application;
using Sucrose.Memory;
using Skylark.Enum;
using Sucrose.Grpc.Common;
using Sucrose.Grpc.Services;
using SGMR = Sucrose.Globalization.Manage.Resources;
using Grpc.Core;

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
        }

        protected void Close()
        {
            Environment.Exit(0);
            Current.Shutdown();
        }

        protected void Configure()
        {
            string Culture = Internal.TrayIconManager.GetSetting("CultureName", SGMR.CultureInfo.Name);
            WindowsThemeType Theme = Internal.TrayIconManager.GetSetting("ThemeType", WindowsThemeType.Dark);

            Internal.TrayIcon.Start(Theme, Culture);

            GeneralServerService.ServerCreate(new List<ServerServiceDefinition>
            {
                Trayer.BindService(new TrayerServerService())
            });

            Internal.TrayIconManager.SetSetting("ThemeType", Theme);
            Internal.TrayIconManager.SetSetting("CultureName", Culture);
            Internal.TrayIconManager.SetSetting("Host", GeneralServerService.Host);
            Internal.TrayIconManager.SetSetting("Port", GeneralServerService.Port);

            GeneralServerService.ServerInstance.Start();
        }

        protected override void OnExit(ExitEventArgs e)
        {
            base.OnExit(e);

            GeneralServerService.ServerInstance.KillAsync().Wait();
            //GeneralServerService.ServerInstance.ShutdownAsync().Wait();

            Internal.TrayIcon.Dispose();

            Close();
        }

        protected override void OnStartup(StartupEventArgs e)
        {
            base.OnStartup(e);

            if (Mutex.WaitOne(TimeSpan.Zero, true))
            {
                Mutex.ReleaseMutex();
                Configure();
            }
            else
            {
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