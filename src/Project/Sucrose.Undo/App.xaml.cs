using Microsoft.Win32;
using System.Diagnostics;
using System.Globalization;
using System.IO;
using System.Media;
using System.Text;
using System.Windows;
using System.Windows.Threading;
using SHC = Skylark.Helper.Culture;
using SMMG = Sucrose.Manager.Manage.General;
using SMMRG = Sucrose.Memory.Manage.Readonly.General;
using SMMRP = Sucrose.Memory.Manage.Readonly.Path;
using SRHR = Sucrose.Resources.Helper.Resources;
using SSDHG = Sucrose.Shared.Dependency.Helper.Graphic;
using SSDHR = Sucrose.Shared.Dependency.Helper.Runtime;
using SUMI = Sucrose.Undo.Manage.Internal;
using SWML = Skylark.Wing.Manage.LockScreen;
using SWUD = Skylark.Wing.Utility.Desktop;

namespace Sucrose.Undo
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

            SHC.All = new CultureInfo(SMMG.Culture, true);

            SSDHR.Configure();

            SSDHG.Configure();
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
                        if (!Record.StartsWith(SUMI.Undo, StringComparison.OrdinalIgnoreCase) && !Record.StartsWith(SUMI.Runtime, StringComparison.OrdinalIgnoreCase))
                        {
                            try
                            {
                                File.Delete(Record);
                            }
                            catch { }
                        }
                    }
                }

                string[] Folders = Directory.GetDirectories(Location, "*", SearchOption.AllDirectories).OrderByDescending(Folder => Folder.Length).ToArray();

                if (Folders.Any())
                {
                    foreach (string Record in Folders)
                    {
                        if (!Record.StartsWith(SUMI.Undo, StringComparison.OrdinalIgnoreCase) && !Record.StartsWith(SUMI.Runtime, StringComparison.OrdinalIgnoreCase))
                        {
                            try
                            {
                                Directory.Delete(Record);
                            }
                            catch { }
                        }
                    }
                }

                if (Location != SUMI.UninstallPath)
                {
                    try
                    {
                        Directory.Delete(Location, true);
                    }
                    catch
                    {
                        try
                        {
                            Directory.Delete(Location);
                        }
                        catch { }
                    }
                }
            }
        }

        private static void TerminateProcess(string Name)
        {
            IEnumerable<Process> Processes = Process.GetProcesses()?.Where(Proc => Proc.ProcessName.Contains(Name) && Proc.Id != Environment.ProcessId);

            foreach (Process Process in Processes)
            {
                try
                {
                    Process?.Kill();
                }
                catch { }
            }
        }

        private static void CleanEmptyParentDirectory(string Location)
        {
            try
            {
                string Parent = Path.GetDirectoryName(Location);
                if (!string.IsNullOrEmpty(Parent) &&
                    !string.Equals(Parent, SMMRP.CommonApplicationData, StringComparison.OrdinalIgnoreCase) &&
                    Directory.Exists(Parent) &&
                    !Directory.EnumerateFileSystemEntries(Parent).Any())
                {
                    Directory.Delete(Parent);
                }
            }
            catch { }
        }

        private static void DeleteSelf(bool CleanAll = false)
        {
            try
            {
                StringBuilder BatchContent = new();

                BatchContent.AppendLine("@echo off");
                BatchContent.AppendLine("setlocal enabledelayedexpansion");
                BatchContent.AppendLine($"taskkill /PID {Environment.ProcessId} /F > nul 2>&1"); /* /T /F */
                BatchContent.AppendLine("timeout /t 3 /nobreak > nul");

                if (CleanAll)
                {
                    BatchContent.AppendLine($@"rd /s /q ""{SUMI.UninstallDataPath}"" > nul 2>&1");
                    BatchContent.AppendLine($@"rd /s /q ""{SUMI.LockScreenDataPath}"" > nul 2>&1");
                }

                BatchContent.AppendLine($@"rd /s /q ""{SUMI.Undo}"" > nul 2>&1");
                BatchContent.AppendLine($@"rd /s /q ""{SUMI.Runtime}"" > nul 2>&1");
                BatchContent.AppendLine($@"rd /s /q ""{SUMI.UninstallPath}"" > nul 2>&1");

                BatchContent.AppendLine(@"del ""%~f0"" > nul 2>&1");
                BatchContent.AppendLine("endlocal");
                BatchContent.AppendLine("exit");

                File.WriteAllText(SUMI.BatchFile, BatchContent.ToString(), Encoding.ASCII);

                Process.Start(new ProcessStartInfo
                {
                    Arguments = $"/c start /B \"\" \"{SUMI.BatchFile}\"",
                    WindowStyle = ProcessWindowStyle.Hidden,
                    WorkingDirectory = SMMRP.Temp,
                    UseShellExecute = true,
                    CreateNoWindow = true,
                    FileName = "cmd.exe"
                });
            }
            catch { }
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

            SRHR.SetLanguage(SMMG.Culture);

            MessageBoxResult Result = MessageBoxResult.Cancel;

            if (!e.Args.Any())
            {
                SystemSounds.Asterisk.Play();

                Result = MessageBox.Show(SUMI.Message, SUMI.Title, MessageBoxButton.YesNoCancel, MessageBoxImage.Question, MessageBoxResult.Cancel);
            }

            if (Result is MessageBoxResult.Yes or MessageBoxResult.No)
            {
                await Task.Delay(SUMI.Delay);

                TerminateProcess(SMMRG.AppName);

                await Task.Delay(SUMI.Delay);

                TerminateProcess(SMMRG.AppName);

                await Task.Delay(SUMI.Delay);

                if (!SWUD.GetDesktopIconVisibility())
                {
                    SWUD.SetDesktopIconVisibility(true);
                }

                SWUD.RefreshDesktop();

                await Task.Delay(SUMI.Delay);

                try
                {
                    SWML.UnregisterProvider();
                }
                catch { }

                await Task.Delay(SUMI.Delay);

                DeleteDirectory(SUMI.UninstallPath);

                if (Result == MessageBoxResult.Yes)
                {
                    await Task.Delay(SUMI.Delay);

                    DeleteDirectory(SUMI.UninstallDataPath);

                    await Task.Delay(SUMI.Delay);

                    DeleteDirectory(SUMI.LockScreenDataPath);

                    CleanEmptyParentDirectory(SUMI.LockScreenDataPath);
                }

                await Task.Delay(SUMI.Delay);

                if (File.Exists(SUMI.Desktop))
                {
                    File.Delete(SUMI.Desktop);
                }

                if (File.Exists(SUMI.StartMenu))
                {
                    File.Delete(SUMI.StartMenu);
                }

                await Task.Delay(SUMI.Delay);

                RegistryKey HomeKey = Registry.CurrentUser.OpenSubKey(SUMI.RegistryName, true);
                HomeKey?.DeleteSubKey(SMMRG.AppName, false);

                await Task.Delay(SUMI.Delay);

                DeleteSelf(Result == MessageBoxResult.Yes);
            }

            Close();
        }

        private void CurrentDomain_UnhandledException(object sender, UnhandledExceptionEventArgs e)
        {
            Exception Exception = (Exception)e.ExceptionObject;

            if (Exception != null)
            {
                SystemSounds.Exclamation.Play();

                MessageBox.Show(Exception.Message + Environment.NewLine + Environment.NewLine + Exception.StackTrace, "Error Information", MessageBoxButton.OK, MessageBoxImage.Error);

                Close();
            }
        }

        private void Current_DispatcherUnhandledException(object sender, DispatcherUnhandledExceptionEventArgs e)
        {
            Exception Exception = e.Exception;

            if (Exception != null)
            {
                SystemSounds.Exclamation.Play();

                MessageBox.Show(Exception.Message + Environment.NewLine + Environment.NewLine + Exception.StackTrace, "Error Information", MessageBoxButton.OK, MessageBoxImage.Error);

                e.Handled = true;

                Close();
            }
        }
    }
}