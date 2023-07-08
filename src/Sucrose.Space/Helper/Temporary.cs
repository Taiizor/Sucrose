using System.IO;
using SSHC = Sucrose.Space.Helper.Command;
using SSHL = Sucrose.Space.Helper.Live;

namespace Sucrose.Space.Helper
{
    internal static class Temporary
    {
        public static void Delete(string Path, string Application)
        {
            try
            {
                SSHL.Kill();
                SSHC.Kill(Application);

                Directory.Delete(Path, true);

                SSHC.Run(Application);
            }
            catch
            {
                try
                {
                    SSHL.Kill();
                    SSHC.Kill(Application);

                    File.Delete(Path);

                    SSHC.Run(Application);
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