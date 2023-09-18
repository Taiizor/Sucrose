﻿using System.Runtime.InteropServices;

namespace Sucrose.Backgroundog.Struct.Data
{
    /// <summary>
    /// 
    /// </summary>
    [StructLayout(LayoutKind.Sequential)]
    public struct CpuStruct
    {
        /// <summary>
        /// 
        /// </summary>
        public float? Min;
        /// <summary>
        /// 
        /// </summary>
        public float? Now;
        /// <summary>
        /// 
        /// </summary>
        public float? Max;
        /// <summary>
        /// 
        /// </summary>
        public string? Name;
    }

    /// <summary>
    /// 
    /// </summary>
    [StructLayout(LayoutKind.Sequential)]
    public struct MemoryStruct
    {
        /// <summary>
        /// 
        /// </summary>
        public string? Name;
        /// <summary>
        /// 
        /// </summary>
        public float? MemoryUsed;
        /// <summary>
        /// 
        /// </summary>
        public float? MemoryLoad;
        /// <summary>
        /// 
        /// </summary>
        public float? MemoryAvailable;
        /// <summary>
        /// 
        /// </summary>
        public float? VirtualMemoryUsed;
        /// <summary>
        /// 
        /// </summary>
        public float? VirtualMemoryLoad;
        /// <summary>
        /// 
        /// </summary>
        public float? VirtualMemoryAvailable;
    }

    /// <summary>
    /// 
    /// </summary>
    [StructLayout(LayoutKind.Sequential)]
    public struct BatteryStruct
    {
        /// <summary>
        /// 
        /// </summary>
        public string? Name;
        /// <summary>
        /// 
        /// </summary>
        public float? Voltage;
        /// <summary>
        /// 
        /// </summary>
        public float? ChargeRate;
        /// <summary>
        /// 
        /// </summary>
        public float? ChargeLevel;
        /// <summary>
        /// 
        /// </summary>
        public float? ChargeCurrent;
        /// <summary>
        /// 
        /// </summary>
        public float? DischargeRate;
        /// <summary>
        /// 
        /// </summary>
        public float? DischargeLevel;
        /// <summary>
        /// 
        /// </summary>
        public float? DischargeCurrent;
        /// <summary>
        /// 
        /// </summary>
        public float? DegradationLevel;
        /// <summary>
        /// 
        /// </summary>
        public float? DesignedCapacity;
        /// <summary>
        /// 
        /// </summary>
        public float? RemainingCapacity;
        /// <summary>
        /// 
        /// </summary>
        public float? FullChargedCapacity;
        /// <summary>
        /// 
        /// </summary>
        public float? ChargeDischargeRate;
        /// <summary>
        /// 
        /// </summary>
        public float? ChargeDischargeCurrent;
        /// <summary>
        /// 
        /// </summary>
        public float? RemainingTimeEstimated;
    }
}