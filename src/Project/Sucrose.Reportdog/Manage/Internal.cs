using System.IO;
using SMMRF = Sucrose.Memory.Manage.Readonly.Folder;
using SMMRG = Sucrose.Memory.Manage.Readonly.General;
using SMMRP = Sucrose.Memory.Manage.Readonly.Path;
using SRHI = Sucrose.Reportdog.Helper.Initialize;
using Timer = System.Threading.Timer;

namespace Sucrose.Reportdog.Manage
{
    internal static class Internal
    {
        public static bool Exit = true;

        public static int AppTime = 1000;

        public static SRHI Initialize = new();

        public static Timer OnlineTimer = null;

        public static Timer AnalyticTimer = null;

        public static FileSystemWatcher Watcher = null;

        public static TimeSpan AnalyticTime = TimeSpan.FromMinutes(10);

        public static TimeSpan OnlineTime = TimeSpan.FromSeconds(SMMRG.Randomise.Next(120, 150));

        public static readonly string Source = Path.Combine(SMMRP.ApplicationData, SMMRG.AppName, SMMRF.Cache, SMMRF.Report);
    }
}