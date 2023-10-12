using System.Diagnostics;
using System.IO;

namespace Sucrose.Shared.Space.Helper
{
    internal static class Processor
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

        public static void Runas(string Application)
        {
            ProcessStartInfo ProcessInfo = new(Application)
            {
                UseShellExecute = true,
                Verb = "runas"
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

        public static void Runas(string Application, string Arguments)
        {
            Runas(Application, Arguments, ProcessWindowStyle.Hidden);
        }

        public static void Run(string Application, string Arguments, ProcessWindowStyle Style)
        {
            Run(Application, Arguments, Style, true);
        }

        public static void Runas(string Application, string Arguments, ProcessWindowStyle Style)
        {
            Runas(Application, Arguments, Style, true);
        }

        public static void Run(string Application, string Arguments, ProcessWindowStyle Style, bool Window)
        {
            ProcessStartInfo ProcessInfo = new(Application, Parse(Arguments))
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

        public static void Runas(string Application, string Arguments, ProcessWindowStyle Style, bool Window)
        {
            ProcessStartInfo ProcessInfo = new(Application, Parse(Arguments))
            {
                CreateNoWindow = Window,
                UseShellExecute = true,
                WindowStyle = Style,
                Verb = "runas"
            };

            Process Process = new()
            {
                StartInfo = ProcessInfo
            };

            Process.Start();
        }

        public static Process Get(string Application)
        {
            if (Work(Application))
            {
                return Process.GetProcessesByName(Path.GetFileNameWithoutExtension(Application)).First();
            }
            else
            {
                return null;
            }
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

        public static bool Work(params string[] Applications)
        {
            return Applications.Any(Work);
        }

        public static int WorkCount(string Application)
        {
            return Process.GetProcessesByName(Path.GetFileNameWithoutExtension(Application)).Count();
        }

        public static int WorkCount(params string[] Applications)
        {
            return Applications.Sum(WorkCount);
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

        public static bool Kill(params string[] Applications)
        {
            return Applications.Any(Kill);
        }

        private static string Parse(string Arguments)
        {
            if (string.IsNullOrEmpty(Arguments))
            {
                Arguments = string.Empty;
            }

            if (!Arguments.StartsWith("\""))
            {
                Arguments = $"\"{Arguments}";
            }

            if (!Arguments.EndsWith("\""))
            {
                Arguments = $"{Arguments}\"";
            }

            return Arguments;
        }

        private static string Parse2(string Arguments)
        {
            return $"\"{Arguments.Trim('\"')}\"";
        }
    }
}