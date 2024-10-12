using System.Diagnostics;
using SMMCA = Sucrose.Memory.Manage.Constant.Aurora;
using SMMI = Sucrose.Manager.Manage.Internal;
using SMMM = Sucrose.Manager.Manage.Manager;
using SMR = Sucrose.Memory.Readonly;
using SSSEL = Sucrose.Shared.Space.Extension.Lifecycle;
using SSSHL = Sucrose.Shared.Space.Helper.Live;
using SSSHM = Sucrose.Shared.Space.Helper.Management;
using SSSHP = Sucrose.Shared.Space.Helper.Processor;
using SWUD = Skylark.Wing.Utility.Desktop;

namespace Sucrose.Shared.Live.Helper
{
    internal static class Kill
    {
        public static void Stop()
        {
            SSSHL.Kill();

            if (!string.IsNullOrEmpty(SMMM.AppProcessName))
            {
                SSSHP.Kill(SMMM.AppProcessName);
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
                    .Where(Process => (Process.ProcessName.Contains(SMR.WebViewProcessName) || Process.ProcessName.Contains(SMR.CefSharpProcessName)) && SSSHM.GetCommandLine(Process).Contains(SMR.AppName))
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