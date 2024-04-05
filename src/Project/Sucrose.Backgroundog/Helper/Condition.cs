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
using SSDEPPT = Sucrose.Shared.Dependency.Enum.PausePerformanceType;
using SSDEPT = Sucrose.Shared.Dependency.Enum.PerformanceType;
using SSLHR = Sucrose.Shared.Live.Helper.Run;
using SSSEL = Sucrose.Shared.Space.Extension.Lifecycle;
using SSSHL = Sucrose.Shared.Space.Helper.Live;
using SSSHM = Sucrose.Shared.Space.Helper.Management;
using SSSHP = Sucrose.Shared.Space.Helper.Processor;
using SSWW = Sucrose.Shared.Watchdog.Watch;

namespace Sucrose.Backgroundog.Helper
{
    internal static class Condition
    {
        public static async Task Start()
        {
            if (SBMI.Performance == SSDEPT.Pause && ((SBMI.App != null && SBMI.App.HasExited) || (SBMI.Live != null && SBMI.Live.HasExited)))
            {
                SBMI.Condition = false;
                SBMI.Performance = SSDEPT.Resume;
                SBMI.NetworkPerformance = SSDENPT.Not;
                SBMI.CategoryPerformance = SSDECPT.Not;
            }
            else
            {
                if (await CpuCondition())
                {
                    return;
                }

                if (await FocusCondition())
                {
                    return;
                }

                if (await SaverCondition())
                {
                    return;
                }

                if (await MemoryCondition())
                {
                    return;
                }

                if (await RemoteCondition())
                {
                    return;
                }

                if (await VirtualCondition())
                {
                    return;
                }

                if (await NetworkCondition())
                {
                    return;
                }

                if (await BatteryCondition())
                {
                    return;
                }

                if (await FullscreenCondition())
                {
                    return;
                }
            }

            await Task.CompletedTask;
        }

        private static async void Lifecycle()
        {
            if (SBMI.Performance == SSDEPT.Close)
            {
                SMMI.BackgroundogSettingManager.SetSetting(SMC.ClosePerformance, false);
                SSLHR.Start();
            }
            else
            {
                SMMI.BackgroundogSettingManager.SetSetting(SMC.PausePerformance, false);
                SBMI.Live = SSSHL.Get();

                if (SBMI.Live != null && !SBMI.Live.HasExited)
                {
                    if (SBMI.PausePerformance == SSDEPPT.Heavy)
                    {
                        SSSEL.Resume(SBMI.Live);

                        if (SMR.WebViewLive.Contains(SBMI.Live.ProcessName) || SMR.CefSharpLive.Contains(SBMI.Live.ProcessName))
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
                                    });
                            }
                            catch (Exception Exception)
                            {
                                await SSWW.Watch_CatchException(Exception);
                            }
                        }
                    }
                }

                if (SBMI.PausePerformance == SSDEPPT.Heavy)
                {
                    if (!string.IsNullOrEmpty(SMMM.App))
                    {
                        SBMI.Apps = SSSHP.Gets(SMMM.App);

                        if (SBMI.Apps != null)
                        {
                            foreach (Process App in SBMI.Apps)
                            {
                                SBMI.App = App;

                                if (App != null && !App.HasExited)
                                {
                                    try
                                    {
                                        SSSEL.Resume(App.MainWindowHandle);
                                        SSSEL.Resume(App.Handle);
                                    }
                                    catch (Exception Exception)
                                    {
                                        await SSWW.Watch_CatchException(Exception);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        private static async Task<bool> CpuCondition()
        {
            if (SBMI.CategoryPerformance == SSDECPT.Cpu)
            {
                int Count = 0;
                int MaxCount = 3;

                while (SMMM.CpuUsage <= 0 || SBMI.CpuData.Now < SMMM.CpuUsage || SBMM.CpuPerformance == SSDEPT.Resume)
                {
                    if (Count >= MaxCount)
                    {
                        Lifecycle();
                        SBMI.Condition = false;
                        SBMI.Performance = SSDEPT.Resume;
                        SBMI.CategoryPerformance = SSDECPT.Not;

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

        private static async Task<bool> FocusCondition()
        {
            if (SBMI.CategoryPerformance == SSDECPT.Focus)
            {
                int Count = 0;
                int MaxCount = 3;

                while (SBMI.FocusDesktop || SBMM.FocusPerformance == SSDEPT.Resume)
                {
                    if (Count >= MaxCount)
                    {
                        Lifecycle();
                        SBMI.Condition = false;
                        SBMI.Performance = SSDEPT.Resume;
                        SBMI.CategoryPerformance = SSDECPT.Not;

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

        private static async Task<bool> SaverCondition()
        {
            if (SBMI.CategoryPerformance == SSDECPT.Saver)
            {
                int Count = 0;
                int MaxCount = 3;

                while (!SBMI.BatteryData.SavingMode || SBMI.BatteryData.SaverStatus == "Off" || SBMM.SaverPerformance == SSDEPT.Resume)
                {
                    if (Count >= MaxCount)
                    {
                        Lifecycle();
                        SBMI.Condition = false;
                        SBMI.Performance = SSDEPT.Resume;
                        SBMI.CategoryPerformance = SSDECPT.Not;

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

        private static async Task<bool> MemoryCondition()
        {
            if (SBMI.CategoryPerformance == SSDECPT.Memory)
            {
                int Count = 0;
                int MaxCount = 3;

                while (SMMM.MemoryUsage <= 0 || SBMI.MemoryData.MemoryLoad < SMMM.MemoryUsage || SBMM.MemoryPerformance == SSDEPT.Resume)
                {
                    if (Count >= MaxCount)
                    {
                        Lifecycle();
                        SBMI.Condition = false;
                        SBMI.Performance = SSDEPT.Resume;
                        SBMI.CategoryPerformance = SSDECPT.Not;

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

        private static async Task<bool> NetworkCondition()
        {
            if (SBMI.CategoryPerformance == SSDECPT.Network)
            {
                int Count = 0;
                int MaxCount = 3;


                if (SBMI.NetworkPerformance == SSDENPT.Ping)
                {
                    while (SMMM.PingValue <= 0 || SBMI.NetworkData.Ping < SMMM.PingValue || SBMM.NetworkPerformance == SSDEPT.Resume)
                    {
                        if (Count >= MaxCount)
                        {
                            Lifecycle();
                            SBMI.Condition = false;
                            SBMI.Performance = SSDEPT.Resume;
                            SBMI.NetworkPerformance = SSDENPT.Not;
                            SBMI.CategoryPerformance = SSDECPT.Not;

                            return true;
                        }
                        else
                        {
                            Count++;
                        }

                        await Task.Delay(TimeSpan.FromSeconds(1));
                    }
                }
                else if (SBMI.NetworkPerformance == SSDENPT.Upload)
                {
                    while (SMMM.UploadValue <= 0 || SBMI.NetworkData.Upload < StorageExtension.Convert(SMMM.UploadValue, SMMM.UploadType, StorageType.Byte, ModeStorageType.Palila) || SBMM.NetworkPerformance == SSDEPT.Resume)
                    {
                        if (Count >= MaxCount)
                        {
                            Lifecycle();
                            SBMI.Condition = false;
                            SBMI.Performance = SSDEPT.Resume;
                            SBMI.NetworkPerformance = SSDENPT.Not;
                            SBMI.CategoryPerformance = SSDECPT.Not;

                            return true;
                        }
                        else
                        {
                            Count++;
                        }

                        await Task.Delay(TimeSpan.FromSeconds(1));
                    }
                }
                else if (SBMI.NetworkPerformance == SSDENPT.Download)
                {
                    while (SMMM.DownloadValue <= 0 || SBMI.NetworkData.Download < StorageExtension.Convert(SMMM.DownloadValue, SMMM.DownloadType, StorageType.Byte, ModeStorageType.Palila) || SBMM.NetworkPerformance == SSDEPT.Resume)
                    {
                        if (Count >= MaxCount)
                        {
                            Lifecycle();
                            SBMI.Condition = false;
                            SBMI.Performance = SSDEPT.Resume;
                            SBMI.NetworkPerformance = SSDENPT.Not;
                            SBMI.CategoryPerformance = SSDECPT.Not;

                            return true;
                        }
                        else
                        {
                            Count++;
                        }

                        await Task.Delay(TimeSpan.FromSeconds(1));
                    }
                }
            }

            return false;
        }

        private static async Task<bool> RemoteCondition()
        {
            if (SBMI.CategoryPerformance == SSDECPT.Remote)
            {
                int Count = 0;
                int MaxCount = 3;

                while (!SBMI.RemoteDesktop || SBMM.RemotePerformance == SSDEPT.Resume)
                {
                    if (Count >= MaxCount)
                    {
                        Lifecycle();
                        SBMI.Condition = false;
                        SBMI.Performance = SSDEPT.Resume;
                        SBMI.CategoryPerformance = SSDECPT.Not;

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

        private static async Task<bool> BatteryCondition()
        {
            if (SBMI.CategoryPerformance == SSDECPT.Battery)
            {
                int Count = 0;
                int MaxCount = 3;

                while (SMMM.BatteryUsage <= 0 || SBMI.BatteryData.PowerLineStatus == PowerLineStatus.Online || SBMI.BatteryData.ACPowerStatus == "Online" || SBMI.BatteryData.ChargeLevel > SMMM.BatteryUsage || SBMM.BatteryPerformance == SSDEPT.Resume)
                {
                    if (Count >= MaxCount)
                    {
                        Lifecycle();
                        SBMI.Condition = false;
                        SBMI.Performance = SSDEPT.Resume;
                        SBMI.CategoryPerformance = SSDECPT.Not;

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

        private static async Task<bool> VirtualCondition()
        {
            if (SBMI.CategoryPerformance == SSDECPT.Virtual)
            {
                int Count = 0;
                int MaxCount = 3;

                while (!SBMI.Virtuality || SBMM.VirtualPerformance == SSDEPT.Resume)
                {
                    if (Count >= MaxCount)
                    {
                        Lifecycle();
                        SBMI.Condition = false;
                        SBMI.Performance = SSDEPT.Resume;
                        SBMI.CategoryPerformance = SSDECPT.Not;

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

        private static async Task<bool> FullscreenCondition()
        {
            if (SBMI.CategoryPerformance == SSDECPT.Fullscreen)
            {
                int Count = 0;
                int MaxCount = 3;

                while (!SBMI.Fullscreen || SBMM.FullscreenPerformance == SSDEPT.Resume)
                {
                    if (Count >= MaxCount)
                    {
                        Lifecycle();
                        SBMI.Condition = false;
                        SBMI.Performance = SSDEPT.Resume;
                        SBMI.CategoryPerformance = SSDECPT.Not;

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