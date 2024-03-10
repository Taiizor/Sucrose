using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Threading.Tasks;

namespace WinUIUnitTests
{
    [TestClass]
    [TestCategory(nameof(WinUIEx.MonitorInfo))]
    public class MonitorInfoTests
    {
        [TestMethod]
        public void GetDisplayMonitors()
        {
            var monitors = MonitorInfo.GetDisplayMonitors().ToList();
            CollectionAssert.AllItemsAreNotNull(monitors);
            Assert.IsTrue(monitors.Any());
            foreach(var monitor in monitors)
            {
                Assert.IsFalse(string.IsNullOrWhiteSpace(monitor.Name));
                Assert.IsTrue(monitor.RectMonitor.Width > 0, "Width");
                Assert.IsTrue(monitor.RectMonitor.Height > 0, "Height");
                Assert.IsTrue(monitor.RectMonitor.Left < monitor.RectMonitor.Right, "Right");
                Assert.IsTrue(monitor.RectMonitor.Top < monitor.RectMonitor.Bottom, "Bottom");
            }
        }
    }
}
