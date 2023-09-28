using Microsoft.Win32;
using System.Diagnostics;
using System.IO;
using System.Windows;
using System.Windows.Threading;

namespace Sucrose.Undo
{
    /// <summary>
    /// Interaction logic for App.xaml
    /// </summary>
    public partial class App : Application
    {
        private static string UninstallPath => Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData), Application);

        private static string StartMenu => Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.StartMenu), "Programs", Shortcut);

        private static string Desktop => Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.Desktop), Shortcut);

        private static string RegistryName => @"Software\Microsoft\Windows\CurrentVersion\Uninstall";

        private static string TemporaryFile => "Sucrose.Backgroundog.sys";

        private static string TemporaryFolder => "Sucrose.Backgroundog";

        private static string Text => "Sucrose Wallpaper Engine";

        private static string Shortcut => $"{Text}.lnk";

        private static string Application => "Sucrose";

        private static int Delay => 1000;

        public App()
        {
            AppDomain.CurrentDomain.UnhandledException += CurrentDomain_UnhandledException;
            Current.DispatcherUnhandledException += Current_DispatcherUnhandledException;
        }

        private static void DeleteDirectory(string Path)
        {
            if (Directory.Exists(Path))
            {
                string[] Files = Directory.GetFiles(Path, "*", SearchOption.AllDirectories);

                foreach (string Record in Files)
                {
                    if (Record == TemporaryFile)
                    {
                        try
                        {
                            File.Delete(Record);
                        }
                        catch { }
                    }
                    else
                    {
                        File.Delete(Record);
                    }
                }

                string[] Folders = Directory.GetDirectories(Path);

                foreach (string Record in Folders)
                {
                    if (Record == TemporaryFolder)
                    {
                        try
                        {
                            Directory.Delete(Record);
                        }
                        catch { }
                    }
                    else
                    {
                        Directory.Delete(Record, true);
                    }
                }

                try
                {
                    Directory.Delete(Path, true);
                }
                catch { }
            }
        }

        private static void TerminateProcess(string Name)
        {
#if NET48_OR_GREATER
            IEnumerable<Process> TerminateProcesses = Process.GetProcesses().Where(Proc => Proc.ProcessName.Contains(Name) && Proc.Id != Process.GetCurrentProcess().Id);
#else
            IEnumerable<Process> TerminateProcesses = Process.GetProcesses().Where(Proc => Proc.ProcessName.Contains(Name) && Proc.Id != Environment.ProcessId);
#endif

            foreach (Process Process in TerminateProcesses)
            {
                Process.Kill();
            }
        }

        protected void Close()
        {
            //Process.GetCurrentProcess().Close();
            //Process.GetCurrentProcess().Kill();
            Environment.Exit(0);
            Current.Shutdown();
            Shutdown();
        }

        protected async override void OnStartup(StartupEventArgs e)
        {
            base.OnStartup(e);

            await Task.Delay(Delay);

            TerminateProcess(Application);

            await Task.Delay(Delay);

            DeleteDirectory(UninstallPath);

            await Task.Delay(Delay);

            RegistryKey HomeKey = Registry.CurrentUser.OpenSubKey(RegistryName, true);
            HomeKey?.DeleteSubKey(Application, false);

            if (File.Exists(Desktop))
            {
                File.Delete(Desktop);
            }

            if (File.Exists(StartMenu))
            {
                File.Delete(StartMenu);
            }

            await Task.Delay(Delay);

            Close();
        }

        private void CurrentDomain_UnhandledException(object sender, UnhandledExceptionEventArgs e)
        {
            Exception Exception = (Exception)e.ExceptionObject;

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