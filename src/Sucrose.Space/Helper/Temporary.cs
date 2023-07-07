using System.IO;

namespace Sucrose.Space.Helper
{
    internal static class Temporary
    {
        public static void Delete(string Path)
        {
            try
            {
                Directory.Delete(Path, true);
            }
            catch
            {
                try
                {
                    File.Delete(Path);
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