using Microsoft.Win32;
using System.Diagnostics;
using System.IO;
using System.Reflection;
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

namespace Sucrose.Bundle
{
    /// <summary>
    /// Interaction logic for Main.xaml
    /// </summary>
    public partial class Main : Window
    {
        private static string PackagePath => Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData), "Package Cache", Application);

        private static string InstallPath => Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData), Application);

        private static string StartMenu => Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.StartMenu), "Programs", Shortcut);

        private static string Desktop => Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.Desktop), Shortcut);

        private static string Uninstall => Path.Combine(PackagePath, "Sucrose.Undo", "Sucrose.Undo.exe");

        private static string RegistryName => @"Software\Microsoft\Windows\CurrentVersion\Uninstall";

        private static string Launcher => Path.Combine(InstallPath, Department, Executable);

        private static string QuietUninstall => $"\"{Uninstall}\" -s";

        private static string Executable => "Sucrose.Launcher.exe";

        private static string Text => "Sucrose Wallpaper Engine";

        private static string Department => "Sucrose.Launcher";

        private static string Shortcut => $"{Text}.lnk";

        private static string Application => "Sucrose";

        private static string Packages => "Packages";

        private static string Publisher => "Taiizor";

        private static string Caches => "Caches";

        private static int MaxDelay => 3000;

        private static int MinDelay => 1000;

        public Main()
        {
            InitializeComponent();
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

        private static async Task ControlDirectory(string Path)
        {
            if (Directory.Exists(Path))
            {
                Directory.Delete(Path, true);
            }

            await Task.Delay(MinDelay);

            Directory.CreateDirectory(Path);
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

            string Size = SHN.Numeral(SSESSE.Convert(File.Length, SEST.Byte, SEST.Kilobyte, SEMST.Toucan), false, false, 0, '0', SECNT.None);

            RegistryKey HomeKey = Registry.CurrentUser.OpenSubKey(RegistryName, true);
            RegistryKey AppKey = HomeKey.CreateSubKey(Application);

            AppKey.SetValue("NoModify", 1, RegistryValueKind.DWord);
            AppKey.SetValue("NoRepair", 1, RegistryValueKind.DWord);
            AppKey.SetValue("DisplayName", Text, RegistryValueKind.String);
            AppKey.SetValue("EstimatedSize", Size, RegistryValueKind.DWord);
            AppKey.SetValue("Publisher", Publisher, RegistryValueKind.String);
            AppKey.SetValue("DisplayIcon", Launcher, RegistryValueKind.String);
            AppKey.SetValue("UninstallString", Uninstall, RegistryValueKind.String);
            AppKey.SetValue("InstallLocation", InstallPath, RegistryValueKind.String);
            AppKey.SetValue("QuietUninstallString", QuietUninstall, RegistryValueKind.String);
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
            await Task.Delay(MinDelay);

            TerminateProcess(Application);

            await Task.Delay(MaxDelay);

            await ControlDirectory(PackagePath);
            await ControlDirectory(InstallPath);

            await Task.Delay(MaxDelay);

            await ExtractResources(Caches, PackagePath);

            await Task.Delay(MinDelay);

            await ExtractResources(Packages, InstallPath);

            await Task.Delay(MaxDelay);

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