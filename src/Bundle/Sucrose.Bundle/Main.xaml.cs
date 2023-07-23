using System.Diagnostics;
using System.IO;
using System.Reflection;
using System.Windows;
using Path = System.IO.Path;
using SEAT = Skylark.Enum.AssemblyType;
using SHA = Skylark.Helper.Assemblies;
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

        private static string Launcher => Path.Combine(Extract, Department, Executable);

        private static string Executable => "Sucrose.Launcher.exe";

        private static string Text => "Sucrose Wallpaper Engine";

        private static string Department => "Sucrose.Launcher";

        private static string Shortcut => $"{Text}.lnk";

        private static string Application => "Sucrose";

        private static string Packages => "Packages";

        private static int MaxDelay => 3000;

        private static int MinDelay => 1000;

        public Main()
        {
            InitializeComponent();
        }

        private static async Task ExtractResources(string SourceFolderPath, string TargetFolderPath)
        {
            if (!Directory.Exists(TargetFolderPath))
            {
                Directory.CreateDirectory(TargetFolderPath);
            }
            else
            {
                Directory.Delete(TargetFolderPath, recursive: true);

                Directory.CreateDirectory(TargetFolderPath);
            }

            await Task.Delay(MaxDelay);

            Assembly Entry = SHA.Assemble(SEAT.Entry);
            string[] Resources = Entry.GetManifestResourceNames();

            foreach (string Resource in Resources)
            {
                if (Resource.StartsWith($"{SourceFolderPath}\\"))
                {
                    string NewResource = Resource.Substring($"{SourceFolderPath}\\".Length);

                    string ExtractFilePath = Path.Combine(TargetFolderPath, NewResource);

                    if (!Directory.Exists(Path.GetDirectoryName(ExtractFilePath)))
                    {
                        Directory.CreateDirectory(Path.GetDirectoryName(ExtractFilePath));
                    }

                    using Stream resourceStream = Entry.GetManifestResourceStream(Resource);
                    using FileStream outputFileStream = new(ExtractFilePath, FileMode.OpenOrCreate);

                    await resourceStream.CopyToAsync(outputFileStream);
                }
            }
        }

        private static void TerminateProcess(string ProcessName)
        {
            IEnumerable<Process> TerminateProcesses = Process.GetProcesses().Where(p => p.ProcessName.Contains(ProcessName) && p.Id != Process.GetCurrentProcess().Id);

            foreach (Process Process in TerminateProcesses)
            {
                Process.Kill();
            }
        }

        private async void Window_ContentRendered(object sender, EventArgs e)
        {
            await Task.Delay(MinDelay);

            TerminateProcess(Application);

            await Task.Delay(MaxDelay);

            await ExtractResources(Packages, Extract);

            await Task.Delay(MaxDelay);

            SWHS.Create(Start, Launcher, null, Path.GetDirectoryName(Launcher), null, Text);
            SWHS.Create(Desktop, Launcher, null, Path.GetDirectoryName(Launcher), null, Text);

            await Task.Delay(MinDelay);

            Process.Start(Launcher);

            await Task.Delay(MinDelay);

            Close();
        }
    }
}