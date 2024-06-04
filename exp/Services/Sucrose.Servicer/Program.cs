using System;
using System.Management;
using System.Runtime.InteropServices;
using System.ServiceProcess;

namespace Sucrose.Servicer
{
    internal static class Program
    {
        /// <summary>
        /// Uygulamanın ana girdi noktası.
        /// </summary>
        static void Main()
        {
            string serviceExePath = @"Sucrose.Servicer.exe";
            string displayName = "Sucrose Hizmetçisi";
            string serviceName = "Sucrose Servicer";

            try
            {
                InstallService(serviceExePath, serviceName, displayName);
                StartService(serviceName);
                Console.WriteLine("Service installed and started successfully.");
            }
            catch (Exception ex)
            {
                Console.WriteLine($"An error occurred: {ex.Message}");
            }



            ServiceBase[] ServicesToRun;
            ServicesToRun = new ServiceBase[]
            {
                new Service1()
            };
            ServiceBase.Run(ServicesToRun);
        }

        static void InstallService(string exePath, string serviceName, string displayName)
        {
            using (ManagementClass managementClass = new ManagementClass("Win32_Service"))
            {
                ManagementBaseObject inParams = managementClass.GetMethodParameters("Create");
                inParams["Name"] = serviceName;
                inParams["DisplayName"] = displayName;
                inParams["PathName"] = exePath;
                inParams["ServiceType"] = 16; // Own Process
                inParams["StartMode"] = "Manual";
                inParams["ErrorControl"] = 1; // Normal
                inParams["StartName"] = null; // LocalSystem
                inParams["StartPassword"] = null;

                ManagementBaseObject outParams = managementClass.InvokeMethod("Create", inParams, null);

                if ((uint)(outParams["ReturnValue"]) != 0)
                {
                    throw new Exception("Failed to create service. Error code: " + outParams["ReturnValue"]);
                }
            }
        }

        static void StartService(string serviceName)
        {
            using (ServiceController serviceController = new ServiceController(serviceName))
            {
                if (serviceController.Status != ServiceControllerStatus.Running)
                {
                    serviceController.Start();
                    serviceController.WaitForStatus(ServiceControllerStatus.Running, TimeSpan.FromSeconds(30));
                }
            }
        }
    }

    public enum ServiceState
    {
        SERVICE_STOPPED = 0x00000001,
        SERVICE_START_PENDING = 0x00000002,
        SERVICE_STOP_PENDING = 0x00000003,
        SERVICE_RUNNING = 0x00000004,
        SERVICE_CONTINUE_PENDING = 0x00000005,
        SERVICE_PAUSE_PENDING = 0x00000006,
        SERVICE_PAUSED = 0x00000007,
    }

    [StructLayout(LayoutKind.Sequential)]
    public struct ServiceStatus
    {
        public int dwServiceType;
        public ServiceState dwCurrentState;
        public int dwControlsAccepted;
        public int dwWin32ExitCode;
        public int dwServiceSpecificExitCode;
        public int dwCheckPoint;
        public int dwWaitHint;
    };
}