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
                { "Max", SBMI.CpuData.Max },
                { "Name", SBMI.CpuData.Name }
            };
        }

        public static JObject GetMemoryInfo()
        {
            return new JObject
            {
                { "Name", SBMI.CpuData.Name },
                { "MemoryUsed", SBMI.MemoryData.MemoryUsed },
                { "MemoryLoad", SBMI.MemoryData.MemoryLoad },
                { "MemoryAvailable", SBMI.MemoryData.MemoryAvailable },
                { "VirtualMemoryUsed", SBMI.MemoryData.VirtualMemoryUsed },
                { "VirtualMemoryLoad", SBMI.MemoryData.VirtualMemoryLoad },
                { "VirtualMemoryAvailable", SBMI.MemoryData.VirtualMemoryAvailable }
            };
        }

        public static JObject GetBatteryInfo()
        {
            return new JObject
            {
                { "Name", SBMI.BatteryData.Name },
                { "Voltage", SBMI.BatteryData.Voltage },
                { "ChargeRate", SBMI.BatteryData.ChargeRate },
                { "ChargeLevel", SBMI.BatteryData.ChargeLevel },
                { "ChargeCurrent", SBMI.BatteryData.ChargeCurrent },
                { "DischargeRate", SBMI.BatteryData.DischargeRate },
                { "DischargeLevel", SBMI.BatteryData.DischargeLevel },
                { "DischargeCurrent", SBMI.BatteryData.DischargeCurrent },
                { "DegradationLevel", SBMI.BatteryData.DegradationLevel },
                { "DesignedCapacity", SBMI.BatteryData.DesignedCapacity },
                { "RemainingCapacity", SBMI.BatteryData.RemainingCapacity },
                { "FullChargedCapacity", SBMI.BatteryData.FullChargedCapacity },
                { "ChargeDischargeRate", SBMI.BatteryData.ChargeDischargeRate },
                { "ChargeDischargeCurrent", SBMI.BatteryData.ChargeDischargeCurrent },
                { "RemainingTimeEstimated", SBMI.BatteryData.RemainingTimeEstimated }
            };
        }
    }
}