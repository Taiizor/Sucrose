using System.Diagnostics;
using SMMA = Sucrose.Manager.Manage.Aurora;
using SMMCA = Sucrose.Memory.Manage.Constant.Aurora;
using SMMI = Sucrose.Manager.Manage.Internal;
using SMR = Sucrose.Memory.Readonly;
using SSSEL = Sucrose.Shared.Space.Extension.Lifecycle;
using SSSHL = Sucrose.Shared.Space.Helper.Live;
using SSSHM = Sucrose.Shared.Space.Helper.Management;
using SSSHP = Sucrose.Shared.Space.Helper.Processor;
using SWUD = Skylark.Wing.Utility.Desktop;
using SMMRG = Sucrose.Memory.Manage.Readonly.General;
using SMMRP = Sucrose.Memory.Manage.Readonly.Process;

namespace Sucrose.Shared.Live.Helper
{
    internal static class Kill
    {
        public static void Stop()
        {
            SSSHL.Kill();

            if (!string.IsNullOrEmpty(SMMA.AppProcessName))
            {
                SSSHP.Kill(SMMA.AppProcessName);
            }

            SWUD.RefreshDesktop();

            SMMI.AuroraSettingManager.SetSetting(SMMCA.AppProcessName, string.Empty);
        }

        public static void StopSubprocess()
        {
            try
            {
                Process[] Processes = Process.GetProcesses();

                Processes
                    .Where(Process => (Process.ProcessName.Contains(SMMRP.WebViewName) || Process.ProcessName.Contains(SMMRP.CefSharpName)) && SSSHM.GetCommandLine(Process).Contains(SMMRG.AppName))
                    .ToList()
                    .ForEach(Process =>
                    {
                        SSSEL.Resume(Process.MainWindowHandle);
                        SSSEL.Resume(Process.Handle);
                        Process.Kill();
                    });
            }
            catch { }
        }
    }
}