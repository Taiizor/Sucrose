using System.IO;
using SBHI = Sucrose.Backgroundog.Helper.Initialize;
using Timer = System.Threading.Timer;
using SMR = Sucrose.Memory.Readonly;

namespace Sucrose.Reportdog.Manage
{
    internal static class Internal
    {
        public static bool Exit = true;

        public static int AppTime = 1000;

        public static SBHI Initialize = new();

        public static Timer InitializeTimer = null;

        public static FileSystemWatcher FileWatcher = null;

        public static readonly string Source = Path.Combine(SMR.AppDataPath, SMR.AppName, SMR.CacheFolder, SMR.ReportFolder);
    }
}