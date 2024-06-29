using System.Diagnostics;
using SMR = Sucrose.Memory.Readonly;
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
                    .Where(Process => Process.ProcessName.Contains(SMR.WebViewProcessName) && SSSHM.GetCommandLine(Process).Contains(SMR.AppName) && !SSEMI.Processes.Contains(Process.Id))
                    .ToList()
                    .ForEach(Process => SSEMI.Processes.Add(Process.Id));
            }
            catch { }
        }
    }
}