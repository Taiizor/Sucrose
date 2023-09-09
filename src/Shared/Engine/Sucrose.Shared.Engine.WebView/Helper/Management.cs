using System.Diagnostics;
using SMR = Sucrose.Memory.Readonly;
using SSEHM = Sucrose.Shared.Engine.Helper.Management;
using SSEWVMI = Sucrose.Shared.Engine.WebView.Manage.Internal;

namespace Sucrose.Shared.Engine.WebView.Helper
{
    internal static class Management
    {
        public static void SetProcesses()
        {
            Process.GetProcesses()
                .Where(Process => Process.ProcessName.Contains(SMR.WebViewProcessName) && SSEHM.GetCommandLine(Process).Contains(SMR.AppName) && !SSEWVMI.Processes.Contains(Process.Id))
                .ToList()
                .ForEach(Process => SSEWVMI.Processes.Add(Process.Id));
        }
    }
}