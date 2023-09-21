using System.IO;
using SSLHK = Sucrose.Shared.Live.Helper.Kill;
using SSSHP = Sucrose.Shared.Space.Helper.Processor;

namespace Sucrose.Shared.Space.Helper
{
    internal static class Temporary
    {
        public static void Delete(string Path, string Application)
        {
            try
            {
                SSLHK.Stop();
                SSSHP.Kill(Application);

                Directory.Delete(Path, true);

                SSSHP.Run(Application);
            }
            catch
            {
                try
                {
                    SSLHK.Stop();
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
    }
}