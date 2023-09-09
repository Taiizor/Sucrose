using System.Diagnostics;
using SMR = Sucrose.Memory.Readonly;
using SSECSMI = Sucrose.Shared.Engine.CefSharp.Manage.Internal;
using SSEHM = Sucrose.Shared.Engine.Helper.Management;

namespace Sucrose.Shared.Engine.CefSharp.Helper
{
    internal static class Management
    {
        public static void SetProcesses()
        {
            Process.GetProcesses()
                .Where(Process => Process.ProcessName.Contains(SMR.CefSharpProcessName) && SSEHM.GetCommandLine(Process).Contains(SMR.AppName) && !SSECSMI.Processes.Contains(Process.Id))
                .ToList()
                .ForEach(Process => SSECSMI.Processes.Add(Process.Id));
        }
    }
}