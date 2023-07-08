using System.Diagnostics;
using System.IO;

namespace Sucrose.Space.Helper
{
    internal static class Command
    {
        public static void Run(string Application)
        {
            ProcessStartInfo ProcessInfo = new(Application)
            {
                UseShellExecute = true
            };

            Process Process = new()
            {
                StartInfo = ProcessInfo
            };

            Process.Start();
        }

        public static void Run(string Application, string Arguments)
        {
            Run(Application, Arguments, ProcessWindowStyle.Hidden);
        }

        public static void Run(string Application, string Arguments, ProcessWindowStyle Style)
        {
            Run(Application, Arguments, Style, true);
        }

        public static void Run(string Application, string Arguments, ProcessWindowStyle Style, bool Window)
        {
            ProcessStartInfo ProcessInfo = new(Application, Arguments)
            {
                CreateNoWindow = Window,
                UseShellExecute = true,
                WindowStyle = Style
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

        public static int WorkCount(string Application)
        {
            return Process.GetProcessesByName(Path.GetFileNameWithoutExtension(Application)).Count();
        }

        public static bool Kill(string Application)
        {
            try
            {
                bool Result = false;

                foreach (Process Process in Process.GetProcessesByName(Path.GetFileNameWithoutExtension(Application)))
                {
                    //Process.CloseMainWindow();
                    //Process.Close();
                    Process.Kill();

                    Result = true;
                }

                return Result;
            }
            catch
            {
                return false;
            }
        }
    }
}