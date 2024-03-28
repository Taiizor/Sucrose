using Microsoft.Win32;
using System.Diagnostics;
using System.IO;
using System.Reflection;
using System.Windows;
using System.Windows.Threading;
using Path = System.IO.Path;
using SEAT = Skylark.Enum.AssemblyType;
using SECNT = Skylark.Enum.ClearNumericType;
using SEMST = Skylark.Enum.ModeStorageType;
using SEST = Skylark.Enum.StorageType;
using SHA = Skylark.Helper.Assemblies;
using SHN = Skylark.Helper.Numeric;
using SSESSE = Skylark.Standard.Extension.Storage.StorageExtension;
using SWHS = Skylark.Wing.Helper.Shortcut;

namespace Sucrose.Bundler
{
    /// <summary>
    /// Interaction logic for App.xaml
    /// </summary>
    public partial class App : Application
    {
        private static string PackagePath => Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData), "Package Cache", Application);

        private static string InstallPath => Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData), Application);

        private static string StartMenu => Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.StartMenu), "Programs", Shortcut);

        private static string Desktop => Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.Desktop), Shortcut);

        private static string Uninstall => Path.Combine(PackagePath, "Sucrose.Undo", "Sucrose.Undo.exe");

        private static string RegistryName => @"Software\Microsoft\Windows\CurrentVersion\Uninstall";

        private static string Wizard => Path.Combine(InstallPath, Department, Executable);

        private static string TemporaryFile => "Sucrose.Backgroundog.sys";

        private static string TemporaryFolder => "Sucrose.Backgroundog";

        private static string QuietUninstall => $"\"{Uninstall}\" -s";

        private static string Executable => "Sucrose.Wizard.exe";

        private static string Text => "Sucrose Wallpaper Engine";

        private static string Department => "Sucrose.Wizard";

        private static string Shortcut => $"{Text}.lnk";

        private static string Application => "Sucrose";

        private static string Packages => "Packages";

        private static string Publisher => "Taiizor";

        private static string Caches => "Caches";

        private static int MaxDelay => 1500;

        private static int MinDelay => 1000;

        private static int Delay => 10;

        public App()
        {
            AppDomain.CurrentDomain.UnhandledException += CurrentDomain_UnhandledException;
            Current.DispatcherUnhandledException += Current_DispatcherUnhandledException;
        }

        private static void SetUninstall()
        {
            Assembly Entry = SHA.Assemble(SEAT.Entry);

#if NET48_OR_GREATER
            FileInfo File = new(Process.GetCurrentProcess().MainModule.FileName);
#else
            FileInfo File = new(Environment.ProcessPath);
#endif

            string Size = SHN.Numeral(SSESSE.Convert(File.Length, SEST.Byte, SEST.Kilobyte, SEMST.Palila), false, false, 0, '0', SECNT.None);

            RegistryKey HomeKey = Registry.CurrentUser.OpenSubKey(RegistryName, true) ?? Registry.CurrentUser.CreateSubKey(RegistryName, true);

            RegistryKey AppKey = HomeKey.CreateSubKey(Application);

            AppKey.SetValue("NoModify", 1, RegistryValueKind.DWord);
            AppKey.SetValue("NoRepair", 1, RegistryValueKind.DWord);
            AppKey.SetValue("DisplayName", Text, RegistryValueKind.String);
            AppKey.SetValue("EstimatedSize", Size, RegistryValueKind.DWord);
            AppKey.SetValue("DisplayIcon", Wizard, RegistryValueKind.String);
            AppKey.SetValue("Publisher", Publisher, RegistryValueKind.String);
            AppKey.SetValue("PublisherName", Publisher, RegistryValueKind.String);
            AppKey.SetValue("UninstallString", Uninstall, RegistryValueKind.String);
            AppKey.SetValue("InstallLocation", InstallPath, RegistryValueKind.String);
            AppKey.SetValue("QuietUninstallString", QuietUninstall, RegistryValueKind.String);
            AppKey.SetValue("BundleVersion", Entry.GetName().Version, RegistryValueKind.String);
            AppKey.SetValue("DisplayVersion", Entry.GetName().Version, RegistryValueKind.String);
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

        private static async Task ControlDirectory(string Location)
        {
            if (Directory.Exists(Location))
            {
                Directory.Delete(Location, true);
            }

            await Task.Delay(MinDelay);

            Directory.CreateDirectory(Location);
        }

        private static async Task ControlDirectoryStable(string Location)
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

                string[] Folders = Directory.GetDirectories(Location);

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
            }

            await Task.Delay(MinDelay);
        }

        private static async Task ExtractResources(string SourcePath, string ExtractPath)
        {
            Assembly Entry = SHA.Assemble(SEAT.Entry);
            string[] Resources = Entry.GetManifestResourceNames();

            foreach (string Resource in Resources)
            {
                if (Resource.StartsWith($"{SourcePath}\\"))
                {
#if NET48_OR_GREATER
                    string Resourcer = Resource.Substring($"{SourcePath}\\".Length);
#else
                    string Resourcer = Resource[$"{SourcePath}\\".Length..];
#endif

                    string ExtractFilePath = Path.Combine(ExtractPath, Resourcer);

                    if (!Directory.Exists(Path.GetDirectoryName(ExtractFilePath)))
                    {
                        Directory.CreateDirectory(Path.GetDirectoryName(ExtractFilePath));
                    }

                    using Stream ResourceStream = Entry.GetManifestResourceStream(Resource);
                    using FileStream OutputFileStream = new(ExtractFilePath, FileMode.OpenOrCreate);

                    await ResourceStream.CopyToAsync(OutputFileStream);

                    await Task.Delay(Delay);
                }
            }
        }

        protected void Close(int Code = 0)
        {
            Environment.Exit(Code);
            Current.Shutdown();
            Shutdown();
        }

        protected async override void OnStartup(StartupEventArgs e)
        {
            base.OnStartup(e);

            await Task.Delay(MinDelay);

            TerminateProcess(Application);

            await Task.Delay(MinDelay);

            TerminateProcess(Application);

            await Task.Delay(MaxDelay);

            await ControlDirectory(PackagePath);
            await ControlDirectoryStable(InstallPath);

            await Task.Delay(MaxDelay);

            await ExtractResources(Caches, PackagePath);

            await Task.Delay(MinDelay);

            await ExtractResources(Packages, InstallPath);

            await Task.Delay(MaxDelay);

            SWHS.Create(Desktop, Wizard, null, Path.GetDirectoryName(Wizard), null, Text);
            SWHS.Create(StartMenu, Wizard, null, Path.GetDirectoryName(Wizard), null, Text);

            await Task.Delay(MinDelay);

            SetUninstall();

            await Task.Delay(MinDelay);

            Process.Start(Wizard);

            await Task.Delay(MinDelay);

            Close();
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