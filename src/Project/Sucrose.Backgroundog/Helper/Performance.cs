using Skylark.Enum;
using Skylark.Standard.Extension.Storage;
using System.Diagnostics;
using SBMI = Sucrose.Backgroundog.Manage.Internal;
using SBMM = Sucrose.Backgroundog.Manage.Manager;
using SMC = Sucrose.Memory.Constant;
using SMMI = Sucrose.Manager.Manage.Internal;
using SMMM = Sucrose.Manager.Manage.Manager;
using SMR = Sucrose.Memory.Readonly;
using SSDECPT = Sucrose.Shared.Dependency.Enum.CategoryPerformanceType;
using SSDENPT = Sucrose.Shared.Dependency.Enum.NetworkPerformanceType;
using SSDEPT = Sucrose.Shared.Dependency.Enum.PerformanceType;
using SSLHK = Sucrose.Shared.Live.Helper.Kill;
using SSSEL = Sucrose.Shared.Space.Extension.Lifecycle;
using SSSHL = Sucrose.Shared.Space.Helper.Live;
using SSSHM = Sucrose.Shared.Space.Helper.Management;
using SSSHP = Sucrose.Shared.Space.Helper.Processor;
using SSWW = Sucrose.Shared.Watchdog.Watch;

namespace Sucrose.Backgroundog.Helper
{
    internal static class Performance
    {
        public static async Task Start()
        {
            if (await CpuPerformance())
            {
                return;
            }

            if (await FocusPerformance())
            {
                return;
            }

            if (await SaverPerformance())
            {
                return;
            }

            if (await MemoryPerformance())
            {
                return;
            }

            if (await RemotePerformance())
            {
                return;
            }

            if (await VirtualPerformance())
            {
                return;
            }

            if (await NetworkPerformance())
            {
                return;
            }

            if (await BatteryPerformance())
            {
                return;
            }

            if (await FullscreenPerformance())
            {
                return;
            }

            await Task.CompletedTask;
        }

        private static async void Lifecycle()
        {
            if (SBMI.Performance == SSDEPT.Close)
            {
                SMMI.BackgroundogSettingManager.SetSetting(SMC.ClosePerformance, true);
                SSLHK.Stop();
            }
            else
            {
                SMMI.BackgroundogSettingManager.SetSetting(SMC.PausePerformance, true);
                SBMI.Live = SSSHL.Get();

                if (SBMI.Live != null && !SBMI.Live.HasExited)
                {
                    SSSEL.Suspend(SBMI.Live);

                    if (SMR.WebViewLive.Contains(SBMI.Live.ProcessName) || SMR.CefSharpLive.Contains(SBMI.Live.ProcessName))
                    {
                        try
                        {
                            Process[] Processes = Process.GetProcesses();

                            Processes
                                .Where(Process => (Process.ProcessName.Contains(SMR.WebViewProcessName) || Process.ProcessName.Contains(SMR.CefSharpProcessName)) && SSSHM.GetCommandLine(Process).Contains(SMR.AppName))
                                .ToList()
                                .ForEach(Process => SSSEL.Suspend(Process));
                        }
                        catch (Exception Exception)
                        {
                            await SSWW.Watch_CatchException(Exception);
                        }
                    }
                }

                if (!string.IsNullOrEmpty(SMMM.App))
                {
                    SBMI.App = SSSHP.Get(SMMM.App);

                    if (SBMI.App != null && !SBMI.App.HasExited)
                    {
                        SSSEL.Suspend(SBMI.App);
                    }
                }
            }
        }

        private static async Task<bool> CpuPerformance()
        {
            if (SBMM.CpuPerformance != SSDEPT.Resume)
            {
                int Count = 0;
                int MaxCount = 5;

                while (SBMI.CpuData.State && SMMM.CpuUsage > 0 && SBMI.CpuData.Now >= SMMM.CpuUsage)
                {
                    if (Count >= MaxCount)
                    {
                        SBMI.Performance = SBMM.CpuPerformance;
                        SBMI.CategoryPerformance = SSDECPT.Cpu;
                        SBMI.Condition = true;
                        Lifecycle();

                        return true;
                    }
                    else
                    {
                        Count++;
                    }

                    await Task.Delay(TimeSpan.FromSeconds(1));
                }
            }

            return false;
        }

        private static async Task<bool> FocusPerformance()
        {
            if (SBMM.FocusPerformance != SSDEPT.Resume)
            {
                int Count = 0;
                int MaxCount = 5;

                while (!SBMI.FocusDesktop)
                {
                    if (Count >= MaxCount)
                    {
                        SBMI.Performance = SBMM.FocusPerformance;
                        SBMI.CategoryPerformance = SSDECPT.Focus;
                        SBMI.Condition = true;
                        Lifecycle();

                        return true;
                    }
                    else
                    {
                        Count++;
                    }

                    await Task.Delay(TimeSpan.FromSeconds(1));
                }
            }

            return false;
        }

        private static async Task<bool> SaverPerformance()
        {
            if (SBMM.SaverPerformance != SSDEPT.Resume)
            {
                int Count = 0;
                int MaxCount = 5;

                while (SBMI.BatteryData.State && (SBMI.BatteryData.SavingMode || SBMI.BatteryData.SaverStatus == "On"))
                {
                    if (Count >= MaxCount)
                    {
                        SBMI.Performance = SBMM.SaverPerformance;
                        SBMI.CategoryPerformance = SSDECPT.Saver;
                        SBMI.Condition = true;
                        Lifecycle();

                        return true;
                    }
                    else
                    {
                        Count++;
                    }

                    await Task.Delay(TimeSpan.FromSeconds(1));
                }
            }

            return false;
        }

        private static async Task<bool> MemoryPerformance()
        {
            if (SBMM.MemoryPerformance != SSDEPT.Resume)
            {
                int Count = 0;
                int MaxCount = 5;

                while (SBMI.MemoryData.State && SMMM.MemoryUsage > 0 && SBMI.MemoryData.MemoryLoad >= SMMM.MemoryUsage)
                {
                    if (Count >= MaxCount)
                    {
                        SBMI.Performance = SBMM.MemoryPerformance;
                        SBMI.CategoryPerformance = SSDECPT.Memory;
                        SBMI.Condition = true;
                        Lifecycle();

                        return true;
                    }
                    else
                    {
                        Count++;
                    }

                    await Task.Delay(TimeSpan.FromSeconds(1));
                }
            }

            return false;
        }

        private static async Task<bool> RemotePerformance()
        {
            if (SBMM.RemotePerformance != SSDEPT.Resume)
            {
                int Count = 0;
                int MaxCount = 5;

                while (SBMI.RemoteDesktop)
                {
                    if (Count >= MaxCount)
                    {
                        SBMI.Performance = SBMM.RemotePerformance;
                        SBMI.CategoryPerformance = SSDECPT.Remote;
                        SBMI.Condition = true;
                        Lifecycle();

                        return true;
                    }
                    else
                    {
                        Count++;
                    }

                    await Task.Delay(TimeSpan.FromSeconds(1));
                }
            }

            return false;
        }

        private static async Task<bool> NetworkPerformance()
        {
            if (SBMM.NetworkPerformance != SSDEPT.Resume)
            {
                int Count = 0;
                int MaxCount = 5;

                while (SBMI.NetworkData.State && (SMMM.PingValue > 0 || SMMM.UploadValue > 0 || SMMM.DownloadValue > 0))
                {
                    if (SBMI.NetworkData.Ping >= SMMM.PingValue)
                    {
                        if (Count >= MaxCount)
                        {
                            SBMI.Performance = SBMM.NetworkPerformance;
                            SBMI.CategoryPerformance = SSDECPT.Network;
                            SBMI.NetworkPerformance = SSDENPT.Ping;
                            SBMI.Condition = true;
                            Lifecycle();

                            return true;
                        }
                        else
                        {
                            Count++;
                        }
                    }
                    else if (SBMI.NetworkData.Upload >= StorageExtension.Convert(SMMM.UploadValue, SMMM.UploadType, StorageType.Byte, ModeStorageType.Palila))
                    {
                        if (Count >= MaxCount)
                        {
                            SBMI.Performance = SBMM.NetworkPerformance;
                            SBMI.CategoryPerformance = SSDECPT.Network;
                            SBMI.NetworkPerformance = SSDENPT.Upload;
                            SBMI.Condition = true;
                            Lifecycle();

                            return true;
                        }
                        else
                        {
                            Count++;
                        }
                    }
                    else if (SBMI.NetworkData.Download >= StorageExtension.Convert(SMMM.DownloadValue, SMMM.DownloadType, StorageType.Byte, ModeStorageType.Palila))
                    {
                        if (Count >= MaxCount)
                        {
                            SBMI.Performance = SBMM.NetworkPerformance;
                            SBMI.CategoryPerformance = SSDECPT.Network;
                            SBMI.NetworkPerformance = SSDENPT.Download;
                            SBMI.Condition = true;
                            Lifecycle();

                            return true;
                        }
                        else
                        {
                            Count++;
                        }
                    }
                    else
                    {
                        break;
                    }

                    await Task.Delay(TimeSpan.FromSeconds(1));
                }
            }

            return false;
        }

        private static async Task<bool> BatteryPerformance()
        {
            if (SBMM.BatteryPerformance != SSDEPT.Resume)
            {
                int Count = 0;
                int MaxCount = 5;

                while (SBMI.BatteryData.State && (SBMI.BatteryData.PowerLineStatus != PowerLineStatus.Online || SBMI.BatteryData.ACPowerStatus != "Online") && SMMM.BatteryUsage > 0 && SBMI.BatteryData.ChargeLevel <= SMMM.BatteryUsage)
                {
                    if (Count >= MaxCount)
                    {
                        SBMI.Performance = SBMM.BatteryPerformance;
                        SBMI.CategoryPerformance = SSDECPT.Battery;
                        SBMI.Condition = true;
                        Lifecycle();

                        return true;
                    }
                    else
                    {
                        Count++;
                    }

                    await Task.Delay(TimeSpan.FromSeconds(1));
                }
            }

            return false;
        }

        private static async Task<bool> VirtualPerformance()
        {
            if (SBMM.VirtualPerformance != SSDEPT.Resume)
            {
                int Count = 0;
                int MaxCount = 5;

                while (SBMI.Virtuality)
                {
                    if (Count >= MaxCount)
                    {
                        SBMI.Performance = SBMM.VirtualPerformance;
                        SBMI.CategoryPerformance = SSDECPT.Virtual;
                        SBMI.Condition = true;
                        Lifecycle();

                        return true;
                    }
                    else
                    {
                        Count++;
                    }

                    await Task.Delay(TimeSpan.FromSeconds(1));
                }
            }

            return false;
        }

        private static async Task<bool> FullscreenPerformance()
        {
            if (SBMM.FullscreenPerformance != SSDEPT.Resume)
            {
                int Count = 0;
                int MaxCount = 5;

                while (SBMI.Fullscreen)
                {
                    if (Count >= MaxCount)
                    {
                        SBMI.Performance = SBMM.FullscreenPerformance;
                        SBMI.CategoryPerformance = SSDECPT.Fullscreen;
                        SBMI.Condition = true;
                        Lifecycle();

                        return true;
                    }
                    else
                    {
                        Count++;
                    }

                    await Task.Delay(TimeSpan.FromSeconds(1));
                }
            }

            return false;
        }
    }
}