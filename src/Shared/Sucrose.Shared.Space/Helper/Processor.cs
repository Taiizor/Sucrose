using System.Diagnostics;
using System.IO;
using SSDSHS = Sucrose.Shared.Dependency.Struct.HandleStruct;

namespace Sucrose.Shared.Space.Helper
{
    internal static class Processor
    {
        public static SSDSHS Run(string Application)
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

            return Result(Process);
        }

        public static SSDSHS Runas(string Application)
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

            return Result(Process);
        }

        public static SSDSHS Run(string Application, string Arguments)
        {
            return Run(Application, Arguments, ProcessWindowStyle.Hidden);
        }

        public static SSDSHS Runas(string Application, string Arguments)
        {
            return Runas(Application, Arguments, ProcessWindowStyle.Hidden);
        }

        public static SSDSHS Run(string Application, string Arguments, ProcessWindowStyle Style)
        {
            return Run(Application, Arguments, Style, true);
        }

        public static SSDSHS Runas(string Application, string Arguments, ProcessWindowStyle Style)
        {
            return Runas(Application, Arguments, Style, true);
        }

        public static SSDSHS Run(string Application, string Arguments, ProcessWindowStyle Style, bool Window)
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

            return Result(Process);
        }

        public static SSDSHS Runas(string Application, string Arguments, ProcessWindowStyle Style, bool Window)
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

            return Result(Process);
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

        private static SSDSHS Result(Process Process)
        {
            try
            {
                Process.WaitForInputIdle();

                return new()
                {
                    Process = Process,
                    Handle = Process.Handle,
                    MainWindowHandle = Process.MainWindowHandle
                };
            }
            catch
            {
                return new();
            }
        }
    }
}