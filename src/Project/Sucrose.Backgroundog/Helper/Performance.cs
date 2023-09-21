using SBEL = Sucrose.Backgroundog.Extension.Lifecycle;
using SBMI = Sucrose.Backgroundog.Manage.Internal;
using SBMM = Sucrose.Backgroundog.Manage.Manager;
using SMMM = Sucrose.Manager.Manage.Manager;
using SSDECPT = Sucrose.Shared.Dependency.Enum.CategoryPerformanceType;
using SSDEPT = Sucrose.Shared.Dependency.Enum.PerformanceType;
using SSLHK = Sucrose.Shared.Live.Helper.Kill;
using SSSHL = Sucrose.Shared.Space.Helper.Live;
using SSSHP = Sucrose.Shared.Space.Helper.Processor;

namespace Sucrose.Backgroundog.Helper
{
    internal static class Performance
    {
        public static async Task Start()
        {
            Console.WriteLine("Run");
            //Performans şartları kontrol edilecek...

            if (await CpuPerformance())
            {
                return;
            }

            await Task.CompletedTask;
        }

        private static void Lifecycle()
        {
            if (SBMI.Performance == SSDEPT.Close)
            {
                SSLHK.Stop();
            }
            else
            {
                SBMI.Live = SSSHL.Get();

                if (SBMI.Live != null && !SBMI.Live.HasExited)
                {
                    SBEL.Suspend(SBMI.Live);
                }

                if (!string.IsNullOrEmpty(SMMM.App))
                {
                    SBMI.App = SSSHP.Get(SMMM.App);

                    if (SBMI.App != null && !SBMI.App.HasExited)
                    {
                        SBEL.Suspend(SBMI.App);
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
    }
}