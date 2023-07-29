using Microsoft.Win32;
using System.Diagnostics;
using System.IO;
using System.Reflection;
using System.Windows;
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
        private static string Extract => Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData), Application);

        private static string Start => Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.StartMenu), "Programs", Shortcut);

        private static string Desktop => Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.Desktop), Shortcut);

        private static string Uninstall => Path.Combine(Extract, "Sucrose.Uninstaller", "Uninstaller.exe");

        private static string Launcher => Path.Combine(Extract, Department, Executable);

        private static string Executable => "Sucrose.Launcher.exe";

        private static string Text => "Sucrose Wallpaper Engine";

        private static string Department => "Sucrose.Launcher";

        private static string Shortcut => $"{Text}.lnk";

        private static string Application => "Sucrose";

        private static string Packages => "Packages";

        private static string Publisher => "Taiizor";

        private static int MaxDelay => 3000;

        private static int MinDelay => 1000;

        public Main()
        {
            InitializeComponent();
        }

        private static void TerminateProcess(string ProcessName)
        {
            IEnumerable<Process> TerminateProcesses = Process.GetProcesses().Where(Proc => Proc.ProcessName.Contains(ProcessName) && Proc.Id != Process.GetCurrentProcess().Id);

            foreach (Process Process in TerminateProcesses)
            {
                Process.Kill();
            }
        }

        private static async Task ExtractDirectory(string ExtractPath)
        {
            if (Directory.Exists(ExtractPath))
            {
                Directory.Delete(ExtractPath, recursive: true);
            }

            await Task.Delay(MinDelay);

            Directory.CreateDirectory(ExtractPath);
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

            FileInfo File = new(Entry.Location);

            string Size = SHN.Numeral(SSESSE.Convert(File.Length, SEST.Byte, SEST.Kilobyte, SEMST.Toucan), false, false, 0, '0', SECNT.None);

            string InstallerRegLoc = @"Software\Microsoft\Windows\CurrentVersion\Uninstall";

            RegistryKey HomeKey = Registry.CurrentUser.OpenSubKey(InstallerRegLoc, true);
            RegistryKey AppKey = HomeKey.CreateSubKey(Application);

            AppKey.SetValue("NoModify", 1, RegistryValueKind.DWord);
            AppKey.SetValue("NoRepair", 1, RegistryValueKind.DWord);
            AppKey.SetValue("DisplayName", Text, RegistryValueKind.String);
            AppKey.SetValue("EstimatedSize", Size, RegistryValueKind.DWord);
            AppKey.SetValue("Publisher", Publisher, RegistryValueKind.String);
            AppKey.SetValue("DisplayIcon", Launcher, RegistryValueKind.String);
            AppKey.SetValue("InstallLocation", Extract, RegistryValueKind.String);
            AppKey.SetValue("UninstallString", Uninstall, RegistryValueKind.String);
            AppKey.SetValue("DisplayVersion", Entry.GetName().Version, RegistryValueKind.String);
        }

        private async void Window_ContentRendered(object sender, EventArgs e)
        {
            await Task.Delay(MinDelay);

            TerminateProcess(Application);

            await Task.Delay(MaxDelay);

            await ExtractDirectory(Extract);

            await Task.Delay(MaxDelay);

            await ExtractResources(Packages, Extract);

            await Task.Delay(MaxDelay);

            SWHS.Create(Start, Launcher, null, Path.GetDirectoryName(Launcher), null, Text);
            SWHS.Create(Desktop, Launcher, null, Path.GetDirectoryName(Launcher), null, Text);

            await Task.Delay(MinDelay);

            SetUninstall();

            await Task.Delay(MinDelay);

            Process.Start(Launcher);

            await Task.Delay(MinDelay);

            Close();
        }
    }
}