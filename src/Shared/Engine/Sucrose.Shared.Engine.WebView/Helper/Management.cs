using System.Diagnostics;
using SMR = Sucrose.Memory.Readonly;
using SSEWVMI = Sucrose.Shared.Engine.WebView.Manage.Internal;
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
                    .Where(Process => Process.ProcessName.Contains(SMR.WebViewProcessName) && SSSHM.GetCommandLine(Process).Contains(SMR.AppName) && !SSEWVMI.Processes.Contains(Process.Id))
                    .ToList()
                    .ForEach(Process => SSEWVMI.Processes.Add(Process.Id));
            }
            catch { }
        }
    }
}