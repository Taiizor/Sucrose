using System.Diagnostics;
using SMMRG = Sucrose.Memory.Manage.Readonly.General;
using SMMRP = Sucrose.Memory.Manage.Readonly.Process;
using SSEMI = Sucrose.Shared.Engine.Manage.Internal;
using SSSHM = Sucrose.Shared.Space.Helper.Management;

namespace Sucrose.Shared.Engine.WebView.Helper
{
    internal static class Management
    {
        public static void SetProcesses()
        {
            try
            {
                Process.GetProcesses()
                    .Where(Process => Process.ProcessName.Contains(SMMRP.WebViewName) && SSSHM.GetCommandLine(Process).Contains(SMMRG.AppName) && !SSEMI.Processes.Contains(Process.Id))
                    .ToList()
                    .ForEach(Process => SSEMI.Processes.Add(Process.Id));
            }
            catch { }
        }
    }
}