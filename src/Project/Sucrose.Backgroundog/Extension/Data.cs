using Newtonsoft.Json.Linq;
using SBMI = Sucrose.Backgroundog.Manage.Internal;

namespace Sucrose.Backgroundog.Extension
{
    internal static class Data
    {
        public static JObject GetCpuInfo()
        {
            return new JObject
            {
                { "Min", SBMI.CpuData.Min },
                { "Now", SBMI.CpuData.Now },
                { "Max", SBMI.CpuData.Max }
            };
        }

        public static JObject GetMemoryInfo()
        {
            return new JObject
            {
                { "MemoryUsed", SBMI.MemoryData.MemoryUsed },
                { "MemoryLoad", SBMI.MemoryData.MemoryLoad },
                { "MemoryAvailable", SBMI.MemoryData.MemoryAvailable },
                { "VirtualMemoryUsed", SBMI.MemoryData.VirtualMemoryUsed },
                { "VirtualMemoryLoad", SBMI.MemoryData.VirtualMemoryLoad },
                { "VirtualMemoryAvailable", SBMI.MemoryData.VirtualMemoryAvailable }
            };
        }
    }
}