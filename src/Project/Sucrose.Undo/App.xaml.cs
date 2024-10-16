using Microsoft.Win32;
using System.Diagnostics;
using System.IO;
using System.Windows;
using System.Windows.Threading;
using SHC = Skylark.Helper.Culture;
using SMMRP = Sucrose.Memory.Manage.Readonly.Path;
using SRER = Sucrose.Resources.Extension.Resources;
using SRHR = Sucrose.Resources.Helper.Resources;

namespace Sucrose.Undo
{
    /// <summary>
    /// Interaction logic for App.xaml
    /// </summary>
    public partial class App : Application
    {
        private static string Message => SRER.GetValue("Undo", "QuestionMessage") + Environment.NewLine + Environment.NewLine + SRER.GetValue("Undo", "QuestionDescription");

        private static string UninstallPath => Path.Combine(SMMRP.LocalApplicationData, Application);

        private static string RegistryName => @"Software\Microsoft\Windows\CurrentVersion\Uninstall";

        private static string UninstallDataPath => Path.Combine(SMMRP.ApplicationData, Application);

        private static string StartMenu => Path.Combine(SMMRP.StartMenu, "Programs", Shortcut);

        private static string Desktop => Path.Combine(SMMRP.Desktop, Shortcut);

        private static string Title => SRER.GetValue("Undo", "QuestionTitle");

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

        private static void DeleteDirectory(string Location)
        {
            if (Directory.Exists(Location))
            {
                string[] Files = Directory.GetFiles(Location, "*", SearchOption.AllDirectories);

                if (Files.Any())
                {
                    foreach (string Record in Files)
                    {
                        if (Path.GetFileName(Record) == TemporaryFile)
                        {
                            try
                            {
                                File.Delete(Record);
                            }
                            catch { }
                        }
                        else
                        {
                            try
                            {
                                File.Delete(Record);
                            }
                            catch { }
                        }
                    }
                }

                string[] Folders = Directory.GetDirectories(Location, "*", SearchOption.AllDirectories);

                if (Folders.Any())
                {
                    foreach (string Record in Folders)
                    {
                        if (Path.GetFileName(Record) == TemporaryFolder)
                        {
                            try
                            {
                                Directory.Delete(Record);
                            }
                            catch { }
                        }
                        else
                        {
                            try
                            {
                                Directory.Delete(Record);
                            }
                            catch { }
                        }
                    }
                }

                try
                {
                    Directory.Delete(Location, true);
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
                try
                {
                    Process.Kill();
                }
                catch { }
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

            MessageBoxResult Result = MessageBoxResult.Cancel;

            SRHR.SetLanguage(SHC.CurrentUITwoLetterISOLanguageName);

            if (!e.Args.Any())
            {
                Result = MessageBox.Show(Message, Title, MessageBoxButton.YesNoCancel, MessageBoxImage.Question);
            }

            if (Result is MessageBoxResult.Yes or MessageBoxResult.No)
            {
                await Task.Delay(Delay);

                TerminateProcess(Application);

                await Task.Delay(Delay);

                TerminateProcess(Application);

                await Task.Delay(Delay);

                DeleteDirectory(UninstallPath);

                if (Result == MessageBoxResult.Yes)
                {
                    await Task.Delay(Delay);

                    DeleteDirectory(UninstallDataPath);
                }

                await Task.Delay(Delay);

                if (File.Exists(Desktop))
                {
                    File.Delete(Desktop);
                }

                if (File.Exists(StartMenu))
                {
                    File.Delete(StartMenu);
                }

                await Task.Delay(Delay);

                RegistryKey HomeKey = Registry.CurrentUser.OpenSubKey(RegistryName, true);
                HomeKey?.DeleteSubKey(Application, false);

                await Task.Delay(Delay);
            }

            Close();
        }

        private void CurrentDomain_UnhandledException(object sender, UnhandledExceptionEventArgs e)
        {
            Exception Exception = (Exception)e.ExceptionObject;

            if (Exception != null)
            {
                MessageBox.Show(Exception.Message + Environment.NewLine + Environment.NewLine + Exception.StackTrace, "Error Information", MessageBoxButton.OK, MessageBoxImage.Error);

                Close();
            }
        }

        private void Current_DispatcherUnhandledException(object sender, DispatcherUnhandledExceptionEventArgs e)
        {
            Exception Exception = e.Exception;

            if (Exception != null)
            {
                MessageBox.Show(Exception.Message + Environment.NewLine + Environment.NewLine + Exception.StackTrace, "Error Information", MessageBoxButton.OK, MessageBoxImage.Error);

                e.Handled = true;

                Close();
            }
        }
    }
}