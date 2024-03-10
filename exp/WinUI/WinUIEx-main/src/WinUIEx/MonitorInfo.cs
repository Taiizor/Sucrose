using System.Runtime.InteropServices;
using Windows.Win32.Foundation;
using Windows.Win32.Graphics.Gdi;
using Windows.Foundation;
using System.Collections.Generic;
using Windows.Win32;

namespace WinUIEx
{
    /// <summary>
    /// Contains information about a display monitor.
    /// </summary>
    public class MonitorInfo
    {
        /// <summary>
        /// Gets the display monitors (including invisible pseudo-monitors associated with the mirroring drivers).
        /// </summary>
        /// <returns>A list of display monitors</returns>
        public unsafe static IList<MonitorInfo> GetDisplayMonitors()
        {
            int monitorCount = PInvoke.GetSystemMetrics(Windows.Win32.UI.WindowsAndMessaging.SYSTEM_METRICS_INDEX.SM_CMONITORS);
            List<MonitorInfo> list = new List<MonitorInfo>(monitorCount);
            MONITORENUMPROC callback = new MONITORENUMPROC((HMONITOR monitor, HDC deviceContext, RECT* rect, LPARAM data) =>
            {
                list.Add(new MonitorInfo(monitor, rect));
                return true;
            });
            LPARAM data = new LPARAM();
            bool ok = PInvoke.EnumDisplayMonitors(null, null, callback, data);
            if (!ok)
                Marshal.ThrowExceptionForHR(Marshal.GetLastWin32Error());
            return list;
        }

        private readonly HMONITOR _monitor;

        internal unsafe MonitorInfo(HMONITOR monitor, RECT* rect)
        {
            RectMonitor =
                new Rect(new Point(rect->left, rect->top),
                new Point(rect->right, rect->bottom));
            _monitor = monitor;
            var info = new MONITORINFOEXW() { __AnonymousBase_winuser_L13558_C43 = new MONITORINFO() { cbSize = (uint)sizeof(MONITORINFOEXW) } };
            GetMonitorInfo(monitor, ref info);
            RectWork =
                new Rect(new Point(info.__AnonymousBase_winuser_L13558_C43.rcWork.left, info.__AnonymousBase_winuser_L13558_C43.rcWork.top),
                new Point(info.__AnonymousBase_winuser_L13558_C43.rcWork.right, info.__AnonymousBase_winuser_L13558_C43.rcWork.bottom));
            Name = new string(info.szDevice.AsSpan()).Replace("\0", "").Trim();
        }

        /// <summary>
        /// Gets the name of the display.
        /// </summary>
        public string Name { get; }

        /// <summary>
        /// Gets the display monitor rectangle, expressed in virtual-screen coordinates.
        /// </summary>
        /// <remarks>
        /// <note>If the monitor is not the primary display monitor, some of the rectangle's coordinates may be negative values.</note>
        /// </remarks>
        public Rect RectMonitor { get; }

        /// <summary>
        /// Gets the work area rectangle of the display monitor, expressed in virtual-screen coordinates.
        /// </summary>
        /// <remarks>
        /// <note>If the monitor is not the primary display monitor, some of the rectangle's coordinates may be negative values.</note>
        /// </remarks>
        public Rect RectWork { get; }

        /// <inheritdoc />
        public override string ToString() => $"{Name} {RectMonitor.Width}x{RectMonitor.Height}";

        private static unsafe bool GetMonitorInfo(HMONITOR hMonitor, ref MONITORINFOEXW lpmi)
        {
            fixed (MONITORINFOEXW* lpmiLocal = &lpmi)

            {
                bool __result = GetMonitorInfo(hMonitor, lpmiLocal);
                return __result;
            }
        }
        [DllImport("User32", ExactSpelling = true, EntryPoint = "GetMonitorInfoW")]
        [DefaultDllImportSearchPaths(DllImportSearchPath.System32)] private static extern unsafe bool GetMonitorInfo(HMONITOR hMonitor, MONITORINFOEXW* lpmi);
    }
}