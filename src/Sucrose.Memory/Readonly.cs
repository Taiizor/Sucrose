using System;

namespace Sucrose.Memory
{
    public static class Readonly
    {
        public static readonly string AppName = "Sucrose";

        public static readonly string CacheFolder = "Cache";

        public static readonly string CefSharp = "CefSharp";

        public static readonly string AppDataPath = Environment.GetFolderPath(Environment.SpecialFolder.ApplicationData);
    }
}