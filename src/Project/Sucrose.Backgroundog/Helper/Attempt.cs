using System.Diagnostics;
using SBEL = Sucrose.Backgroundog.Extension.Lifecycle;
using SBMI = Sucrose.Backgroundog.Manage.Internal;
using SMR = Sucrose.Memory.Readonly;
using SSSHL = Sucrose.Shared.Space.Helper.Live;
using SSSHM = Sucrose.Shared.Space.Helper.Management;

namespace Sucrose.Backgroundog.Helper
{
    internal static class Attempt
    {
        public static async Task Start()
        {
            int MaxAttempts = 5;
            int IntervalSeconds = 1;

            for (int Attempt = 0; Attempt < MaxAttempts; Attempt++)
            {
                if (SSSHL.Run())
                {
                    return;
                }

                await Task.Delay(TimeSpan.FromSeconds(IntervalSeconds));
            }

            Process.GetProcesses()
                .Where(Process => (Process.ProcessName.Contains(SMR.WebViewProcessName) || Process.ProcessName.Contains(SMR.CefSharpProcessName)) && SSSHM.GetCommandLine(Process).Contains(SMR.AppName))
                .ToList()
                .ForEach(Process => SBEL.Resume(Process));

            SBMI.Exit = false;
            SBMI.Initialize.Stop();

            await Task.CompletedTask;
        }
    }
}