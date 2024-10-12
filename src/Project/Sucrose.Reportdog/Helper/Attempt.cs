using System.Diagnostics;
using SMR = Sucrose.Memory.Readonly;
using SRMI = Sucrose.Reportdog.Manage.Internal;
using SMMRG = Sucrose.Memory.Manage.Readonly.General;

namespace Sucrose.Reportdog.Helper
{
    internal static class Attempt
    {
        public static async Task Start()
        {
            int MaxAttempt = 5;
            int IntervalSecond = 1;

            for (int Attempt = 0; Attempt < MaxAttempt; Attempt++)
            {
#if NET48_OR_GREATER
                IEnumerable<Process> Processes = Process.GetProcesses().Where(Proc => Proc.ProcessName.Contains(SMMRG.AppName) && Proc.Id != Process.GetCurrentProcess().Id);
#else
                IEnumerable<Process> Processes = Process.GetProcesses().Where(Proc => Proc.ProcessName.Contains(SMMRG.AppName) && Proc.Id != Environment.ProcessId);
#endif

                if (Processes.Any())
                {
                    await Task.Delay(TimeSpan.FromSeconds(3));

                    return;
                }

                await Task.Delay(TimeSpan.FromSeconds(IntervalSecond));
            }

            SRMI.Exit = false;
            SRMI.Initialize.Stop();

            await Task.CompletedTask;
        }
    }
}