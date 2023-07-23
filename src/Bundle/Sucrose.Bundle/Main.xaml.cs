using Microsoft.Win32;
using System.IO;
using System.Reflection;
using System.Windows;
using Path = System.IO.Path;

namespace Sucrose.Bundle
{
    /// <summary>
    /// Interaction logic for Main.xaml
    /// </summary>
    public partial class Main : Window
    {
        private static readonly string Application = "Sucrose";

        private static readonly string Shortcut = "Sucrose Wallpaper Engine.lnk";

        //private static readonly string Launcher = Path.Combine(Extract, "Sucrose.Launcher", "Sucrose.Launcher.exe");

        private static readonly string Desktop = Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.Desktop), Shortcut);

        private static readonly string Extract = Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData), Application);

        public Main()
        {
            InitializeComponent();
            //MessageBox.Show(Launcher);
        }

        private static void CreateShortcut(string shortcutPath, string targetPath)
        {
            IWshRuntimeLibrary.WshShell shell = new();
            if (shell.CreateShortcut(shortcutPath) is IWshRuntimeLibrary.IWshShortcut shortcut)
            {
                shortcut.TargetPath = targetPath;
                shortcut.Save();
            }
        }

        private static void SetRunAtStartup()
        {
            string appName = "MyApp";
            string executablePath = AppContext.BaseDirectory; //Assembly.GetExecutingAssembly().Location
            RegistryKey key = Registry.CurrentUser.OpenSubKey("SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Run", true);
            key.SetValue(appName, executablePath);
        }

        private static void ExtractEmbeddedResources(string sourceFolderPath, string targetFolderPath)
        {
            if (!Directory.Exists(targetFolderPath))
            {
                Directory.CreateDirectory(targetFolderPath);
            }

            Assembly assembly = Assembly.GetEntryAssembly();
            string[] resourceNames = assembly.GetManifestResourceNames();

            foreach (string resourceName in resourceNames)
            {
                if (resourceName.StartsWith($"{sourceFolderPath}\\"))
                {
                    string newResourceName = resourceName.Substring($"{sourceFolderPath}\\".Length);

                    string outputFilePath = Path.Combine(targetFolderPath, newResourceName);

                    if (!Directory.Exists(Path.GetDirectoryName(outputFilePath)))
                    {
                        Directory.CreateDirectory(Path.GetDirectoryName(outputFilePath));
                    }

                    using Stream resourceStream = assembly.GetManifestResourceStream(resourceName);
                    using FileStream outputFileStream = new(outputFilePath, FileMode.OpenOrCreate);
                    resourceStream.CopyTo(outputFileStream);
                }
            }
        }

        private void Window_ContentRendered(object sender, EventArgs e)
        {
            //Task.Delay(3000).Wait();

            //ExtractEmbeddedResources("Files", Extract);

            //Task.Delay(3000).Wait();

            //CreateShortcut(Shortcut, Launcher);

            //Task.Delay(3000).Wait();

            //Close();
        }
    }
}