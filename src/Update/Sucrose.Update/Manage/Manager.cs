using System.IO;
using SMR = Sucrose.Memory.Readonly;

namespace Sucrose.Update.Manage
{
    internal static class Manager
    {
        public static string CachePath => Path.Combine(SMR.AppDataPath, SMR.AppName, SMR.CacheFolder, SMR.Bundle);
    }
}