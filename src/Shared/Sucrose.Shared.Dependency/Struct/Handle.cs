using System.Diagnostics;
using System.Runtime.InteropServices;

namespace Sucrose.Shared.Dependency.Struct
{
    /// <summary>
    /// 
    /// </summary>
    [StructLayout(LayoutKind.Sequential)]
    public struct HandleStruct
    {
        /// <summary>
        /// 
        /// </summary>
        public IntPtr Handle;
        /// <summary>
        /// 
        /// </summary>
        public Process Process;
        /// <summary>
        /// 
        /// </summary>
        public IntPtr MainWindowHandle;
    }
}