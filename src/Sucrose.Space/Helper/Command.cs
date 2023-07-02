using System.Diagnostics;

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
                CreateNoWindow = true,
                UseShellExecute = false
            };

            Process Process = new()
            {
                StartInfo = ProcessInfo
            };

            Process.Start();
        }
    }
}