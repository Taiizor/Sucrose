using Skylark.Enum;
using Skylark.Standard.Extension.Storage;
using System.Diagnostics;
using SBEG = Sucrose.Backgroundog.Extension.Graphic;
using SBMI = Sucrose.Backgroundog.Manage.Internal;
using SMMA = Sucrose.Manager.Manage.Aurora;
using SMMB = Sucrose.Manager.Manage.Backgroundog;
using SMMCB = Sucrose.Memory.Manage.Constant.Backgroundog;
using SMMI = Sucrose.Manager.Manage.Internal;
using SMMRA = Sucrose.Memory.Manage.Readonly.App;
using SMR = Sucrose.Memory.Readonly;
using SSDECPT = Sucrose.Shared.Dependency.Enum.CategoryPerformanceType;
using SSDENPT = Sucrose.Shared.Dependency.Enum.NetworkPerformanceType;
using SSDEPPT = Sucrose.Shared.Dependency.Enum.PausePerformanceType;
using SSDEPT = Sucrose.Shared.Dependency.Enum.PerformanceType;
using SSDMMB = Sucrose.Shared.Dependency.Manage.Manager.Backgroundog;
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
                SMMI.BackgroundogSettingManager.SetSetting(SMMCB.ClosePerformance, true);
                SSLHK.Stop();
            }
            else
            {
                SMMI.BackgroundogSettingManager.SetSetting(SMMCB.PausePerformance, true);
                SBMI.PausePerformance = SSDMMB.PausePerformanceType;
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
                    if (!string.IsNullOrEmpty(SMMA.AppProcessName))
                    {
                        SBMI.Apps = SSSHP.Gets(SMMA.AppProcessName);

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
            if (SSDMMB.CpuPerformance != SSDEPT.Resume)
            {
                int Count = 0;
                int MaxCount = 5;
                SSDEPT Performance = SSDMMB.CpuPerformance;

                while (SBMI.CpuData.State && SMMB.CpuUsage > 0 && SBMI.CpuData.Now >= SMMB.CpuUsage && SSDMMB.CpuPerformance == Performance)
                {
                    if (Count >= MaxCount)
                    {
                        SBMI.Performance = SSDMMB.CpuPerformance;
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
            if (SSDMMB.GpuPerformance != SSDEPT.Resume)
            {
                int Count = 0;
                int MaxCount = 5;
                SSDEPT Performance = SSDMMB.GpuPerformance;

                while (SBMI.GraphicData.State && SMMB.GpuUsage > 0 && SBEG.Performance() && SSDMMB.GpuPerformance == Performance)
                {
                    if (Count >= MaxCount)
                    {
                        SBMI.Performance = SSDMMB.GpuPerformance;
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
            if (SSDMMB.FocusPerformance != SSDEPT.Resume)
            {
                int Count = 0;
                int MaxCount = 5;
                SSDEPT Performance = SSDMMB.FocusPerformance;

                while (!SBMI.FocusDesktop && SSDMMB.FocusPerformance == Performance)
                {
                    if (Count >= MaxCount)
                    {
                        SBMI.Performance = SSDMMB.FocusPerformance;
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
            if (SSDMMB.SaverPerformance != SSDEPT.Resume)
            {
                int Count = 0;
                int MaxCount = 5;
                SSDEPT Performance = SSDMMB.SaverPerformance;

                while (SBMI.BatteryData.State && (SBMI.BatteryData.SavingMode || SBMI.BatteryData.SaverStatus == "On") && SSDMMB.SaverPerformance == Performance)
                {
                    if (Count >= MaxCount)
                    {
                        SBMI.Performance = SSDMMB.SaverPerformance;
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
            if (SSDMMB.MemoryPerformance != SSDEPT.Resume)
            {
                int Count = 0;
                int MaxCount = 5;
                SSDEPT Performance = SSDMMB.MemoryPerformance;

                while (SBMI.MemoryData.State && SMMB.MemoryUsage > 0 && SBMI.MemoryData.MemoryLoad >= SMMB.MemoryUsage && SSDMMB.MemoryPerformance == Performance)
                {
                    if (Count >= MaxCount)
                    {
                        SBMI.Performance = SSDMMB.MemoryPerformance;
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
            if (SSDMMB.RemotePerformance != SSDEPT.Resume)
            {
                int Count = 0;
                int MaxCount = 5;
                SSDEPT Performance = SSDMMB.RemotePerformance;

                while (SBMI.RemoteDesktop && SSDMMB.RemotePerformance == Performance)
                {
                    if (Count >= MaxCount)
                    {
                        SBMI.Performance = SSDMMB.RemotePerformance;
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
            if (SSDMMB.NetworkPerformance != SSDEPT.Resume)
            {
                int Count = 0;
                int MaxCount = 5;
                SSDEPT Performance = SSDMMB.NetworkPerformance;

                while (SBMI.NetworkData.State && (SMMB.PingValue > 0 || SMMB.UploadValue > 0 || SMMB.DownloadValue > 0) && SSDMMB.NetworkPerformance == Performance)
                {
                    if (SBMI.NetworkData.Ping >= SMMB.PingValue)
                    {
                        if (Count >= MaxCount)
                        {
                            SBMI.Performance = SSDMMB.NetworkPerformance;
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
                    else if (SBMI.NetworkData.Upload >= StorageExtension.Convert(SMMB.UploadValue, SMMB.UploadType, StorageType.Byte, ModeStorageType.Palila))
                    {
                        if (Count >= MaxCount)
                        {
                            SBMI.Performance = SSDMMB.NetworkPerformance;
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
                    else if (SBMI.NetworkData.Download >= StorageExtension.Convert(SMMB.DownloadValue, SMMB.DownloadType, StorageType.Byte, ModeStorageType.Palila))
                    {
                        if (Count >= MaxCount)
                        {
                            SBMI.Performance = SSDMMB.NetworkPerformance;
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
            if (SSDMMB.BatteryPerformance != SSDEPT.Resume)
            {
                int Count = 0;
                int MaxCount = 5;
                SSDEPT Performance = SSDMMB.BatteryPerformance;

                while (SBMI.BatteryData.State && (SBMI.BatteryData.PowerLineStatus != PowerLineStatus.Online || SBMI.BatteryData.ACPowerStatus != "Online") && SMMB.BatteryUsage > 0 && SBMI.BatteryData.ChargeLevel <= SMMB.BatteryUsage && SSDMMB.BatteryPerformance == Performance)
                {
                    if (Count >= MaxCount)
                    {
                        SBMI.Performance = SSDMMB.BatteryPerformance;
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
            if (SSDMMB.VirtualPerformance != SSDEPT.Resume)
            {
                int Count = 0;
                int MaxCount = 5;
                SSDEPT Performance = SSDMMB.VirtualPerformance;

                while (SBMI.Virtuality && SSDMMB.VirtualPerformance == Performance)
                {
                    if (Count >= MaxCount)
                    {
                        SBMI.Performance = SSDMMB.VirtualPerformance;
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
            if (SSDMMB.FullscreenPerformance != SSDEPT.Resume)
            {
                int Count = 0;
                int MaxCount = 5;
                SSDEPT Performance = SSDMMB.FullscreenPerformance;

                while (SBMI.Fullscreen && SSDMMB.FullscreenPerformance == Performance)
                {
                    if (Count >= MaxCount)
                    {
                        SBMI.Performance = SSDMMB.FullscreenPerformance;
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