using Microsoft.Win32;
using System.Diagnostics;
using System.IO;
using System.Reflection;
using System.Runtime.InteropServices;
using System.Windows;
using System.Windows.Input;
using Path = System.IO.Path;
using SEAT = Skylark.Enum.AssemblyType;
using SECNT = Skylark.Enum.ClearNumericType;
using SEMST = Skylark.Enum.ModeStorageType;
using SEST = Skylark.Enum.StorageType;
using SHA = Skylark.Helper.Assemblies;
using SHN = Skylark.Helper.Numeric;
using SSESSE = Skylark.Standard.Extension.Storage.StorageExtension;
using SWHS = Skylark.Wing.Helper.Shortcut;
using SWHWI = Skylark.Wing.Helper.WindowInterop;
using SWNM = Skylark.Wing.Native.Methods;

namespace Sucrose.Bundle
{
    /// <summary>
    /// Interaction logic for Main.xaml
    /// </summary>
    public partial class Main : Window
    {
        private static string Description => "Sucrose Wallpaper Engine is a versatile wallpaper engine that brings life to your desktop with a wide range of interactive themes.";

        private static string PackagePath => Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData), "Package Cache", Application);

        private static string InstallPath => Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData), Application);

        private static string StartMenu => Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.StartMenu), "Programs", Shortcut);

        private static string Desktop => Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.Desktop), Shortcut);

        private static string AppDataPath => Environment.GetFolderPath(Environment.SpecialFolder.ApplicationData);

        private static string Uninstall => Path.Combine(PackagePath, "Sucrose.Undo", "Sucrose.Undo.exe");

        private static string RegistryName => @"Software\Microsoft\Windows\CurrentVersion\Uninstall";

        private static string Launcher => Path.Combine(InstallPath, Department, Executable);

        private static string ThemesPath => Path.Combine(AppDataPath, Application, Themes);

        private static string RedistPath => Path.Combine(Path.GetTempPath(), Redist);

        private static string Url => "https://github.com/Taiizor/Sucrose";

        private static string TemporaryFile => "Sucrose.Backgroundog.sys";

        private static string TemporaryFolder => "Sucrose.Backgroundog";

        private static string QuietUninstall => $"\"{Uninstall}\" -s";

        private static string Executable => "Sucrose.Launcher.exe";

        private static string Text => "Sucrose Wallpaper Engine";

        private static string Department => "Sucrose.Launcher";

        private static string Contact => "taiizor@vegalya.com";

        private static string Shortcut => $"{Text}.lnk";

        private static string Application => "Sucrose";

        private static string Packages => "Packages";

        private static string Publisher => "Taiizor";

        private static string Themes => "Themes";

        private static string Redist => "Redist";

        private static string Caches => "Caches";

        private static int MaxDelay => 3000;

        private static int MinDelay => 1000;

        private static int Delay => 10;

        private bool Silent;

        public Main(bool Silent)
        {
            this.Silent = Silent;
            InitializeComponent();
        }

        private void WindowCorner()
        {
            try
            {
                if (!Silent)
                {
                    SWNM.DWMWINDOWATTRIBUTE Attribute = SWNM.DWMWINDOWATTRIBUTE.WindowCornerPreference;
                    SWNM.DWM_WINDOW_CORNER_PREFERENCE Preference = SWNM.DWM_WINDOW_CORNER_PREFERENCE.DWMWCP_ROUND;

                    SWNM.DwmSetWindowAttribute(SWHWI.Handle(this), Attribute, ref Preference, (uint)Marshal.SizeOf(typeof(uint)));
                }
            }
            catch
            {
                //
            }
        }

        private static async Task InstallRedist()
        {
            string Command = $"/q /norestart";

#if X86
            string Executable = Path.Combine(RedistPath, "VC_Redist_x86.exe");
#elif X64
            string Executable = Path.Combine(RedistPath, "VC_Redist_x64.exe");
#else
            string Executable = Path.Combine(RedistPath, "VC_Redist_arm64.exe");
#endif

            ProcessStartInfo Starter = new()
            {
                Arguments = Command,
                FileName = Executable,
                CreateNoWindow = true,
                UseShellExecute = false,
                RedirectStandardError = false,
                RedirectStandardOutput = false
            };

            using (Process Installer = new())
            {
                Installer.StartInfo = Starter;
                Installer.Start();

                Installer.WaitForExit();
            }

            await Task.CompletedTask;
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
            AppKey.SetValue("Contact", Contact, RegistryValueKind.String);
            AppKey.SetValue("DisplayName", Text, RegistryValueKind.String);
            AppKey.SetValue("URLInfoAbout", Url, RegistryValueKind.String);
            AppKey.SetValue("EstimatedSize", Size, RegistryValueKind.DWord);
            AppKey.SetValue("URLUpdateInfo", Url, RegistryValueKind.String);
            AppKey.SetValue("Publisher", Publisher, RegistryValueKind.String);
            AppKey.SetValue("Comments", Description, RegistryValueKind.String);
            AppKey.SetValue("DisplayIcon", Launcher, RegistryValueKind.String);
            AppKey.SetValue("PublisherName", Publisher, RegistryValueKind.String);
            AppKey.SetValue("UninstallString", Uninstall, RegistryValueKind.String);
            AppKey.SetValue("InstallLocation", InstallPath, RegistryValueKind.String);
            AppKey.SetValue("QuietUninstallString", QuietUninstall, RegistryValueKind.String);
            AppKey.SetValue("BundleVersion", Entry.GetName().Version, RegistryValueKind.String);
            AppKey.SetValue("DisplayVersion", Entry.GetName().Version, RegistryValueKind.String);
        }

        private void Window_MouseDown(object sender, MouseButtonEventArgs e)
        {
            if (e.ChangedButton == MouseButton.Left)
            {
                Cursor = Cursors.SizeAll;
                DragMove();
                Cursor = Cursors.Arrow;
            }
        }

        private async void Window_ContentRendered(object sender, EventArgs e)
        {
            WindowCorner();

            await Task.Delay(MinDelay);

            TerminateProcess(Application);

            await Task.Delay(MinDelay);

            TerminateProcess(Application);

            await Task.Delay(MaxDelay);

            await ControlDirectory(RedistPath);
            await ControlDirectory(ThemesPath);
            await ControlDirectory(PackagePath);
            await ControlDirectoryStable(InstallPath);

            await Task.Delay(MaxDelay);

            await ExtractResources(Redist, RedistPath);

            await Task.Delay(MinDelay);

            await ExtractResources(Themes, ThemesPath);

            await Task.Delay(MinDelay);

            await ExtractResources(Caches, PackagePath);

            await Task.Delay(MinDelay);

            await ExtractResources(Packages, InstallPath);

            await Task.Delay(MaxDelay);

            await InstallRedist();

            await Task.Delay(MinDelay);

            SWHS.Create(Desktop, Launcher, null, Path.GetDirectoryName(Launcher), null, Text);
            SWHS.Create(StartMenu, Launcher, null, Path.GetDirectoryName(Launcher), null, Text);

            await Task.Delay(MinDelay);

            SetUninstall();

            await Task.Delay(MinDelay);

            Process.Start(Launcher);

            await Task.Delay(MinDelay);

            Close();
        }
    }
}