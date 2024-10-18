using Microsoft.Win32;
using SharpCompress.Archives;
using SharpCompress.Common;
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
using SHV = Skylark.Helper.Versionly;
using SSESSE = Skylark.Standard.Extension.Storage.StorageExtension;
using SWHSB = Skylark.Wing.Helper.ShortcutBasic;
using SWHSD = Skylark.Wing.Helper.ShortcutDefault;
using SWHWI = Skylark.Wing.Helper.WindowInterop;
using SWNM = Skylark.Wing.Native.Methods;
using SWUD = Skylark.Wing.Utility.Desktop;

#if NET6_0_OR_GREATER
using SWHSR = Skylark.Wing.Helper.ShortcutRuntime;
#endif

namespace Sucrose.Bundle
{
    /// <summary>
    /// Interaction logic for Main.xaml
    /// </summary>
    public partial class Main : Window
    {
        private static string Description => "Sucrose Wallpaper Engine is a versatile wallpaper engine that brings life to your desktop with a wide range of interactive wallpapers.";

        private static string LocalApplicationData => Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData);

        private static string ApplicationData => Environment.GetFolderPath(Environment.SpecialFolder.ApplicationData);

        private static string PackagePath => Path.Combine(LocalApplicationData, "Package Cache", Application);

        private static string StartMenu => Environment.GetFolderPath(Environment.SpecialFolder.StartMenu);

        private static string Uninstall => Path.Combine(PackagePath, "Sucrose.Undo", "Sucrose.Undo.exe");

        private static string Desktop => Environment.GetFolderPath(Environment.SpecialFolder.Desktop);

        private static string StartMenuProgramsShortcut => Path.Combine(StartMenuPrograms, Shortcut);

        private static string RegistryName => @"Software\Microsoft\Windows\CurrentVersion\Uninstall";

        private static string ShowcasePath => Path.Combine(ApplicationData, Application, Showcase);

        private static string PackagesFilePath => Path.Combine(PackagesPath, $"{Application}.7z");

        private static string InstallPath => Path.Combine(LocalApplicationData, Application);

        private static string Launcher => Path.Combine(InstallPath, Department, Executable);

        private static string SevenZipPath => Path.Combine(Path.GetTempPath(), SevenZip);

        private static string PackagesPath => Path.Combine(Path.GetTempPath(), Packages);

        private static string StartMenuPrograms => Path.Combine(StartMenu, "Programs");

        private static string DesktopShortcut => Path.Combine(Desktop, Shortcut);

        private static string Url => "https://github.com/Taiizor/Sucrose";

        private static string TemporaryFile => "Sucrose.Backgroundog.sys";

        private static string TemporaryFolder => "Sucrose.Backgroundog";

        private static string QuietUninstall => $"\"{Uninstall}\" -s";

        private static string Version => $"{SHV.Auto(SEAT.Entry)}";

        private static string Executable => "Sucrose.Launcher.exe";

        private static string Text => "Sucrose Wallpaper Engine";

        private static string Department => "Sucrose.Launcher";

        private static string Contact => "taiizor@vegalya.com";

        private static string Shortcut => $"{Text}.lnk";

        private static string Application => "Sucrose";

        private static string Showcase => "Showcase";

        private static string SevenZip => "SevenZip";

        private static string Publisher => "Taiizor";

        private static string Packages => "Packages";

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

        private async Task WindowCorner()
        {
            try
            {
                if (!Silent)
                {
                    SWNM.DWMWINDOWATTRIBUTE Attribute = SWNM.DWMWINDOWATTRIBUTE.WindowCornerPreference;
                    SWNM.DWM_WINDOW_CORNER_PREFERENCE Preference = SWNM.DWM_WINDOW_CORNER_PREFERENCE.DWMWCP_ROUND;

                    SWNM.DwmSetWindowAttribute(SWHWI.Handle(this), Attribute, ref Preference, (uint)Marshal.SizeOf(typeof(uint)));
                }

                await Task.CompletedTask;
            }
            catch { }
        }

        private static async Task ExtractAll()
        {
            try
            {
                await ExtractArchive();
            }
            catch
            {
                try
                {
                    await ExtractPackages();
                }
                catch
                {
                    await ExtractArchive(Path.Combine(Packages, $"{Application}.7z"), InstallPath);
                }
            }

            await Task.Delay(MinDelay);

            if (File.Exists(Launcher))
            {
                if (Directory.Exists(Path.GetDirectoryName(DesktopShortcut)))
                {
                    bool CreateDesktopShortcut = true;

#if NET6_0_OR_GREATER
                    try
                    {
                        SWHSR.Create(Path.GetDirectoryName(DesktopShortcut), Shortcut, Text, null, Launcher, null, Path.GetDirectoryName(Launcher), null, SWNM.WindowStyle.Normal);

                        CreateDesktopShortcut = false;
                    }
                    catch { }
#endif

                    if (CreateDesktopShortcut)
                    {
                        try
                        {
                            SWHSB.Create(DesktopShortcut, Launcher, Path.GetDirectoryName(Launcher), null, null, SWNM.ShortcutWindowStyles.WshNormalFocus, Text, 0);
                        }
                        catch
                        {
                            SWHSD.Create(DesktopShortcut, Launcher, null, Path.GetDirectoryName(Launcher), null, Text);
                        }
                    }
                }

                if (Directory.Exists(Path.GetDirectoryName(StartMenuProgramsShortcut)))
                {
                    bool CreateStartMenuShortcut = true;

#if NET6_0_OR_GREATER
                    try
                    {
                        SWHSR.Create(Path.GetDirectoryName(StartMenuProgramsShortcut), Shortcut, Text, null, Launcher, null, Path.GetDirectoryName(Launcher), null, SWNM.WindowStyle.Normal);

                        CreateStartMenuShortcut = false;
                    }
                    catch { }
#endif

                    if (CreateStartMenuShortcut)
                    {
                        try
                        {
                            SWHSB.Create(StartMenuProgramsShortcut, Launcher, Path.GetDirectoryName(Launcher), null, null, SWNM.ShortcutWindowStyles.WshNormalFocus, Text, 0);
                        }
                        catch
                        {
                            SWHSD.Create(StartMenuProgramsShortcut, Launcher, null, Path.GetDirectoryName(Launcher), null, Text);
                        }
                    }
                }
            }
        }

        private static async Task SetUninstall()
        {
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
            AppKey.SetValue("BundleVersion", Version, RegistryValueKind.String);
            AppKey.SetValue("DisplayVersion", Version, RegistryValueKind.String);
            AppKey.SetValue("PublisherName", Publisher, RegistryValueKind.String);
            AppKey.SetValue("UninstallString", Uninstall, RegistryValueKind.String);
            AppKey.SetValue("InstallLocation", InstallPath, RegistryValueKind.String);
            AppKey.SetValue("QuietUninstallString", QuietUninstall, RegistryValueKind.String);

            await Task.CompletedTask;
        }

        private static async Task ExtractArchive()
        {
            string Command = $"x \"{PackagesFilePath}\" -o\"{InstallPath}\" -aoa";

            string Executable = Path.Combine(SevenZipPath, "7z.exe");

            ProcessStartInfo Starter = new()
            {
                Arguments = Command,
                CreateNoWindow = true,
                FileName = Executable,
                UseShellExecute = false,
                RedirectStandardError = false,
                RedirectStandardOutput = false,
                WindowStyle = ProcessWindowStyle.Hidden
            };

            using Process Extactor = new()
            {
                EnableRaisingEvents = true,
                StartInfo = Starter
            };

            TaskCompletionSource<bool> Completion = new();

            Extactor.Exited += (sender, args) =>
            {
                Completion.TrySetResult(true);
            };

            Extactor.Start();

            await Completion.Task;
        }

        private static async Task RefreshDesktop()
        {
            try
            {
                SWUD.RefreshDesktop();

                await Task.CompletedTask;
            }
            catch { }
        }

        private static async Task ExtractPackages()
        {
            await Task.Factory.StartNew(() =>
            {
                using IArchive Archiver = ArchiveFactory.Open(PackagesFilePath);

                foreach (IArchiveEntry Record in Archiver.Entries)
                {
                    if (Record.IsDirectory)
                    {
                        if (Directory.Exists(Path.Combine(InstallPath, Record.Key)))
                        {
                            Directory.CreateDirectory(Path.Combine(InstallPath, Record.Key));
                        }
                    }

                    Record.WriteToDirectory(InstallPath, new ExtractionOptions()
                    {
                        PreserveAttributes = true,
                        PreserveFileTime = true,
                        ExtractFullPath = true,
                        Overwrite = true
                    });
                }
            });
        }

        private static async Task TerminateProcess(string Name)
        {
#if NET48_OR_GREATER
            IEnumerable<Process> Processes = Process.GetProcesses().Where(Proc => Proc.ProcessName.Contains(Name) && Proc.Id != Process.GetCurrentProcess().Id);
#else
            IEnumerable<Process> Processes = Process.GetProcesses().Where(Proc => Proc.ProcessName.Contains(Name) && Proc.Id != Environment.ProcessId);
#endif

            foreach (Process Process in Processes)
            {
                try
                {
                    Process.Kill();
                }
                catch { }
            }

            await Task.CompletedTask;
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

        private static async Task ExtractArchive(string SourcePath, string ExtractPath)
        {
            await Task.Factory.StartNew(() =>
            {
                if (!Directory.Exists(ExtractPath))
                {
                    Directory.CreateDirectory(ExtractPath);
                }

                Assembly Entry = SHA.Assemble(SEAT.Entry);

                using IArchive Archive = ArchiveFactory.Open(Entry.GetManifestResourceStream(SourcePath));

                foreach (IArchiveEntry Record in Archive.Entries)
                {
                    if (Record.IsDirectory)
                    {
                        if (Directory.Exists(Path.Combine(ExtractPath, Record.Key)))
                        {
                            Directory.CreateDirectory(Path.Combine(ExtractPath, Record.Key));
                        }
                    }

                    Record.WriteToDirectory(ExtractPath, new ExtractionOptions()
                    {
                        PreserveAttributes = true,
                        PreserveFileTime = true,
                        ExtractFullPath = true,
                        Overwrite = true
                    });
                }
            });
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
            await WindowCorner();

            await Task.Delay(MinDelay);

            await TerminateProcess(Application);

            await Task.Delay(MinDelay);

            await TerminateProcess(Application);

            await Task.Delay(MinDelay);

            await RefreshDesktop();

            await Task.Delay(MaxDelay);

            await ControlDirectory(PackagePath);
            await ControlDirectory(PackagesPath);
            await ControlDirectory(SevenZipPath);
            await ControlDirectory(ShowcasePath);
            await ControlDirectoryStable(InstallPath);

            await Task.Delay(MaxDelay);

            await ExtractResources(Caches, PackagePath);

            await Task.Delay(MinDelay);

            await ExtractResources(Packages, PackagesPath);

            await Task.Delay(MinDelay);

            await ExtractResources(SevenZip, SevenZipPath);

            await Task.Delay(MinDelay);

            await ExtractResources(Showcase, ShowcasePath);

            await Task.Delay(MinDelay);

            await ExtractAll();

            await Task.Delay(MinDelay);

            await SetUninstall();

            await Task.Delay(MinDelay);

            Process.Start(Launcher);

            await Task.Delay(MinDelay);

            Close();
        }
    }
}