using System.IO;
using SMR = Sucrose.Memory.Readonly;

namespace Sucrose.Wizard.Manage
{
    internal static class Manager
    {
        public static string CachePath => Path.Combine(SMR.AppDataPath, SMR.AppName, SMR.CacheFolder, SMR.Bundle);
    }
}