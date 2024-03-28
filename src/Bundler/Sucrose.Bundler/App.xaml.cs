using System.Windows;
using System.Windows.Threading;

namespace Sucrose.Bundler
{
    /// <summary>
    /// Interaction logic for App.xaml
    /// </summary>
    public partial class App : Application
    {
        public App()
        {
            AppDomain.CurrentDomain.UnhandledException += CurrentDomain_UnhandledException;
            Current.DispatcherUnhandledException += Current_DispatcherUnhandledException;
        }

        protected void Close(int Code = 0)
        {
            Environment.Exit(Code);
            Current.Shutdown();
            Shutdown();
        }

        private void CurrentDomain_UnhandledException(object sender, UnhandledExceptionEventArgs e)
        {
            Exception Exception = e.ExceptionObject as Exception;

            if (Exception != null)
            {
                MessageBox.Show(Exception.Message + Environment.NewLine + Exception.StackTrace, "Error", MessageBoxButton.OK, MessageBoxImage.Error);
                Close();
            }
        }

        private void Current_DispatcherUnhandledException(object sender, DispatcherUnhandledExceptionEventArgs e)
        {
            Exception Exception = e.Exception;

            if (Exception != null)
            {
                MessageBox.Show(Exception.Message + Environment.NewLine + Exception.StackTrace, "Error", MessageBoxButton.OK, MessageBoxImage.Error);
                e.Handled = true;
                Close();
            }
        }
    }
}