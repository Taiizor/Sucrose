using System.Diagnostics;
using System.IO;

namespace Sucrose.Space.Helper
{
    internal static class Command
    {
        public static void Run(string Application)
        {
            Run(Application, string.Empty);
        }

        public static void Run(string Application, string Arguments)
        {
            ProcessStartInfo ProcessInfo = new(Application, Arguments)
            {
                WindowStyle = ProcessWindowStyle.Hidden,
                UseShellExecute = true,
                CreateNoWindow = true
            };

            Process Process = new()
            {
                StartInfo = ProcessInfo
            };

            Process.Start();
        }

        public static bool Work(string Application)
        {
            if (Process.GetProcessesByName(Path.GetFileNameWithoutExtension(Application)).Any())
            {
                return true;
            }
            else
            {
                return false;
            }
        }

        public static bool Kill(string Application)
        {
            try
            {
                foreach (Process Process in Process.GetProcessesByName(Path.GetFileNameWithoutExtension(Application)))
                {
                    //Process.CloseMainWindow();
                    //Process.Close();
                    Process.Kill();

                    return true;
                }

                return false;
            }
            catch
            {
                return false;
            }
        }
    }
}