using SBEL = Sucrose.Backgroundog.Extension.Lifecycle;
using SBMI = Sucrose.Backgroundog.Manage.Internal;
using SBMM = Sucrose.Backgroundog.Manage.Manager;
using SMMM = Sucrose.Manager.Manage.Manager;
using SSDECPT = Sucrose.Shared.Dependency.Enum.CategoryPerformanceType;
using SSDEPT = Sucrose.Shared.Dependency.Enum.PerformanceType;
using SSDENPT = Sucrose.Shared.Dependency.Enum.NetworkPerformanceType;
using SSLHR = Sucrose.Shared.Live.Helper.Run;
using SSSHL = Sucrose.Shared.Space.Helper.Live;
using SSSHP = Sucrose.Shared.Space.Helper.Processor;

namespace Sucrose.Backgroundog.Helper
{
    internal static class Condition
    {
        public static async Task Start()
        {
            Console.WriteLine("Condition");

            if (SBMI.Performance == SSDEPT.Pause && ((SBMI.App != null && SBMI.App.HasExited) || (SBMI.Live != null && SBMI.Live.HasExited)))
            {
                SBMI.Condition = false;
                SBMI.Performance = SSDEPT.Resume;
                SBMI.CategoryPerformance = SSDECPT.Not;
            }
            else
            {
                if (await CpuCondition())
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

                if (await BatteryCondition())
                {
                    return;
                }
            }

            await Task.CompletedTask;
        }

        private static void Lifecycle()
        {
            if (SBMI.Performance == SSDEPT.Close)
            {
                SSLHR.Start();
            }
            else
            {
                SBMI.Live = SSSHL.Get();

                if (SBMI.Live != null && !SBMI.Live.HasExited)
                {
                    SBEL.Resume(SBMI.Live);
                }

                if (!string.IsNullOrEmpty(SMMM.App))
                {
                    SBMI.App = SSSHP.Get(SMMM.App);

                    if (SBMI.App != null && !SBMI.App.HasExited)
                    {
                        SBEL.Resume(SBMI.App);
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

                while (SBMI.CpuData.Now < SMMM.CpuUsage || SBMM.CpuPerformance == SSDEPT.Resume)
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

                while (SBMI.MemoryData.MemoryLoad < SMMM.MemoryUsage || SBMM.MemoryPerformance == SSDEPT.Resume)
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

                while (SBMI.BatteryData.ChargeLevel > SMMM.BatteryUsage || SBMM.BatteryPerformance == SSDEPT.Resume)
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