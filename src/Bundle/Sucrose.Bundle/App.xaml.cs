using System.Windows;

namespace Sucrose.Bundle
{
    /// <summary>
    /// Interaction logic for App.xaml
    /// </summary>
    public partial class App : Application
    {
        public App()
        {
            AppDomain.CurrentDomain.UnhandledException += CurrentDomain_UnhandledException;
        }

        private void CurrentDomain_UnhandledException(object sender, UnhandledExceptionEventArgs e)
        {
            Exception Exception = (Exception)e.ExceptionObject;

            MessageBox.Show(Exception.Message + Environment.NewLine + Exception.StackTrace);

            Environment.Exit(0);
            Current.Shutdown();
            Shutdown();
        }
    }
}