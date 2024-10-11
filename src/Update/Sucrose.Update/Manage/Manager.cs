using System.IO;
using SMMRP = Sucrose.Memory.Manage.Readonly.Path;
using SMR = Sucrose.Memory.Readonly;

namespace Sucrose.Update.Manage
{
    internal static class Manager
    {
        public static string CachePath => Path.Combine(SMMRP.ApplicationData, SMR.AppName, SMR.CacheFolder, SMR.Bundle);
    }
}