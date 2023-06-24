using System.Diagnostics;

namespace Sucrose.Space.Helper
{
    internal class Command
    {
        public static void Run(string Application)
        {
            Run(Application, string.Empty);
        }

        public static void Run(string Application, string Arguments)
        {
            ProcessStartInfo ProcessInfo = new(Application, Arguments)
            {
                CreateNoWindow = true, // Konsol penceresini oluşturma
                UseShellExecute = false // Shell kullanmayı devre dışı bırakma
            };

            Process Process = new()
            {
                StartInfo = ProcessInfo
            };

            Process.Start();
        }
    }
}