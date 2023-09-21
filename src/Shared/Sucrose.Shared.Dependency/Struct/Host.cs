using System.Runtime.InteropServices;

namespace Sucrose.Shared.Dependency.Struct.Host
{
    /// <summary>
    /// 
    /// </summary>
    [StructLayout(LayoutKind.Sequential)]
    public struct HostStruct
    {
        /// <summary>
        /// 
        /// </summary>
        public string Name;
        /// <summary>
        /// 
        /// </summary>
        public string Address;
    }
}