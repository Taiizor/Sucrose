using System.IO;
using SSSHL = Sucrose.Shared.Space.Helper.Live;
using SSSHP = Sucrose.Shared.Space.Helper.Processor;

namespace Sucrose.Shared.Space.Helper
{
    internal static class Temporary
    {
        public static void Delete(string Path, string Application)
        {
            try
            {
                SSSHL.Kill();
                SSSHP.Kill(Application);

                Directory.Delete(Path, true);

                SSSHP.Run(Application);
            }
            catch
            {
                try
                {
                    SSSHL.Kill();
                    SSSHP.Kill(Application);

                    File.Delete(Path);

                    SSSHP.Run(Application);
                }
                catch
                {
                    //
                }
            }
        }

        public static bool Check(string Path)
        {
            if (Directory.Exists(Path))
            {
                return true;
            }
            else if (File.Exists(Path))
            {
                return true;
            }
            else
            {
                return false;
            }
        }

        public static long Size(string Path)
        {
            try
            {
                if (Directory.Exists(Path))
                {
                    string[] cacheFiles = Directory.GetFiles(Path, "*.*", SearchOption.AllDirectories);

                    return cacheFiles.Sum(cacheFile => new FileInfo(cacheFile).Length);
                }
                else if (File.Exists(Path))
                {
                    return new FileInfo(Path).Length;
                }
                else
                {
                    return 0;
                }
            }
            catch
            {
                return 0;
            }
        }
    }
}