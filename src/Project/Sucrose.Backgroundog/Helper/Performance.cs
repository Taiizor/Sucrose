using Skylark.Enum;
using Skylark.Standard.Extension.Storage;
using System.Diagnostics;
using SBEG = Sucrose.Backgroundog.Extension.Graphic;
using SBMI = Sucrose.Backgroundog.Manage.Internal;
using SMC = Sucrose.Memory.Constant;
using SMMI = Sucrose.Manager.Manage.Internal;
using SMMM = Sucrose.Manager.Manage.Manager;
using SMR = Sucrose.Memory.Readonly;
using SSDECPT = Sucrose.Shared.Dependency.Enum.CategoryPerformanceType;
using SSDENPT = Sucrose.Shared.Dependency.Enum.NetworkPerformanceType;
using SSDEPPT = Sucrose.Shared.Dependency.Enum.PausePerformanceType;
using SSDEPT = Sucrose.Shared.Dependency.Enum.PerformanceType;
using SSDMM = Sucrose.Shared.Dependency.Manage.Manager;
using SSLHK = Sucrose.Shared.Live.Helper.Kill;
using SSSEL = Sucrose.Shared.Space.Extension.Lifecycle;
using SSSHL = Sucrose.Shared.Space.Helper.Live;
using SSSHM = Sucrose.Shared.Space.Helper.Management;
using SSSHP = Sucrose.Shared.Space.Helper.Processor;
using SSWW = Sucrose.Shared.Watchdog.Watch;
using SMMRM = Sucrose.Memory.Manage.Readonly.Mutex;
using SMMRA = Sucrose.Memory.Manage.Readonly.App;

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

            if (await GpuPerformance())
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
                SBMI.PausePerformance = SSDMM.PausePerformanceType;
                SBMI.Live = SSSHL.Get();

                if (SBMI.Live != null && !SBMI.Live.HasExited)
                {
                    if (SBMI.PausePerformance == SSDEPPT.Heavy)
                    {
                        SSSEL.Suspend(SBMI.Live);

                        if (SMMRA.WebViewLive.Contains(SBMI.Live.ProcessName) || SMMRA.CefSharpLive.Contains(SBMI.Live.ProcessName))
                        {
                            try
                            {
                                Process[] Processes = Process.GetProcesses();

                                Processes
                                    .Where(Process => (Process.ProcessName.Contains(SMR.WebViewProcessName) || Process.ProcessName.Contains(SMR.CefSharpProcessName)) && SSSHM.GetCommandLine(Process).Contains(SMR.AppName))
                                    .ToList()
                                    .ForEach(Process =>
                                    {
                                        SSSEL.Suspend(Process.MainWindowHandle);
                                        SSSEL.Suspend(Process.Handle);
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
                                        SSSEL.Suspend(App.MainWindowHandle);
                                        SSSEL.Suspend(App.Handle);
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

        private static async Task<bool> CpuPerformance()
        {
            if (SSDMM.CpuPerformance != SSDEPT.Resume)
            {
                int Count = 0;
                int MaxCount = 5;
                SSDEPT Performance = SSDMM.CpuPerformance;

                while (SBMI.CpuData.State && SMMM.CpuUsage > 0 && SBMI.CpuData.Now >= SMMM.CpuUsage && SSDMM.CpuPerformance == Performance)
                {
                    if (Count >= MaxCount)
                    {
                        SBMI.Performance = SSDMM.CpuPerformance;
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

        private static async Task<bool> GpuPerformance()
        {
            if (SSDMM.GpuPerformance != SSDEPT.Resume)
            {
                int Count = 0;
                int MaxCount = 5;
                SSDEPT Performance = SSDMM.GpuPerformance;

                while (SBMI.GraphicData.State && SMMM.GpuUsage > 0 && SBEG.Performance() && SSDMM.GpuPerformance == Performance)
                {
                    if (Count >= MaxCount)
                    {
                        SBMI.Performance = SSDMM.GpuPerformance;
                        SBMI.CategoryPerformance = SSDECPT.Gpu;
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
            if (SSDMM.FocusPerformance != SSDEPT.Resume)
            {
                int Count = 0;
                int MaxCount = 5;
                SSDEPT Performance = SSDMM.FocusPerformance;

                while (!SBMI.FocusDesktop && SSDMM.FocusPerformance == Performance)
                {
                    if (Count >= MaxCount)
                    {
                        SBMI.Performance = SSDMM.FocusPerformance;
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
            if (SSDMM.SaverPerformance != SSDEPT.Resume)
            {
                int Count = 0;
                int MaxCount = 5;
                SSDEPT Performance = SSDMM.SaverPerformance;

                while (SBMI.BatteryData.State && (SBMI.BatteryData.SavingMode || SBMI.BatteryData.SaverStatus == "On") && SSDMM.SaverPerformance == Performance)
                {
                    if (Count >= MaxCount)
                    {
                        SBMI.Performance = SSDMM.SaverPerformance;
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
            if (SSDMM.MemoryPerformance != SSDEPT.Resume)
            {
                int Count = 0;
                int MaxCount = 5;
                SSDEPT Performance = SSDMM.MemoryPerformance;

                while (SBMI.MemoryData.State && SMMM.MemoryUsage > 0 && SBMI.MemoryData.MemoryLoad >= SMMM.MemoryUsage && SSDMM.MemoryPerformance == Performance)
                {
                    if (Count >= MaxCount)
                    {
                        SBMI.Performance = SSDMM.MemoryPerformance;
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
            if (SSDMM.RemotePerformance != SSDEPT.Resume)
            {
                int Count = 0;
                int MaxCount = 5;
                SSDEPT Performance = SSDMM.RemotePerformance;

                while (SBMI.RemoteDesktop && SSDMM.RemotePerformance == Performance)
                {
                    if (Count >= MaxCount)
                    {
                        SBMI.Performance = SSDMM.RemotePerformance;
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
            if (SSDMM.NetworkPerformance != SSDEPT.Resume)
            {
                int Count = 0;
                int MaxCount = 5;
                SSDEPT Performance = SSDMM.NetworkPerformance;

                while (SBMI.NetworkData.State && (SMMM.PingValue > 0 || SMMM.UploadValue > 0 || SMMM.DownloadValue > 0) && SSDMM.NetworkPerformance == Performance)
                {
                    if (SBMI.NetworkData.Ping >= SMMM.PingValue)
                    {
                        if (Count >= MaxCount)
                        {
                            SBMI.Performance = SSDMM.NetworkPerformance;
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
                            SBMI.Performance = SSDMM.NetworkPerformance;
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
                            SBMI.Performance = SSDMM.NetworkPerformance;
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
            if (SSDMM.BatteryPerformance != SSDEPT.Resume)
            {
                int Count = 0;
                int MaxCount = 5;
                SSDEPT Performance = SSDMM.BatteryPerformance;

                while (SBMI.BatteryData.State && (SBMI.BatteryData.PowerLineStatus != PowerLineStatus.Online || SBMI.BatteryData.ACPowerStatus != "Online") && SMMM.BatteryUsage > 0 && SBMI.BatteryData.ChargeLevel <= SMMM.BatteryUsage && SSDMM.BatteryPerformance == Performance)
                {
                    if (Count >= MaxCount)
                    {
                        SBMI.Performance = SSDMM.BatteryPerformance;
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
            if (SSDMM.VirtualPerformance != SSDEPT.Resume)
            {
                int Count = 0;
                int MaxCount = 5;
                SSDEPT Performance = SSDMM.VirtualPerformance;

                while (SBMI.Virtuality && SSDMM.VirtualPerformance == Performance)
                {
                    if (Count >= MaxCount)
                    {
                        SBMI.Performance = SSDMM.VirtualPerformance;
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
            if (SSDMM.FullscreenPerformance != SSDEPT.Resume)
            {
                int Count = 0;
                int MaxCount = 5;
                SSDEPT Performance = SSDMM.FullscreenPerformance;

                while (SBMI.Fullscreen && SSDMM.FullscreenPerformance == Performance)
                {
                    if (Count >= MaxCount)
                    {
                        SBMI.Performance = SSDMM.FullscreenPerformance;
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