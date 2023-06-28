using System.Windows;
using Sucrose.Memory;

namespace Sucrose.WPF.UI
{
    /// <summary>
    /// Interaction logic for App.xaml
    /// </summary>
    public partial class App : Application
    {
        private static readonly Mutex Mutex = new(true, Readonly.UserInterfaceMutex);

        public App()
        {
            AppDomain.CurrentDomain.UnhandledException += GlobalUnhandledExceptionHandler;
        }

        protected void Close()
        {
            Environment.Exit(0);
            Current.Shutdown();
            Shutdown();
        }

        protected override void OnExit(ExitEventArgs e)
        {
            base.OnExit(e);

            Close();
        }

        protected override void OnStartup(StartupEventArgs e)
        {
            base.OnStartup(e);

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

        protected void GlobalUnhandledExceptionHandler(object sender, UnhandledExceptionEventArgs e)
        {
            Exception ex = (Exception)e.ExceptionObject;
            MessageBox.Show(ex.Message + "\n" + ex.InnerException + "\n" + ex.StackTrace + "\n" + ex.Data + "\n" + ex.TargetSite + "\n" + ex.HResult + "\n" + ex.Source + "\n" + ex.HelpLink);
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