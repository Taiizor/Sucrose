using System.Runtime.InteropServices;

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
}