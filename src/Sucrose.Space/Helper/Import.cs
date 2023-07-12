using System.IO;
using SMR = Sucrose.Memory.Readonly;
using SSHL = Sucrose.Space.Helper.Live;
using SSHP = Sucrose.Space.Helper.Processor;

namespace Sucrose.Space.Helper
{
    internal static class Import
    {
        public static void Start(string Destination, string Application)
        {
            try
            {
                SSHL.Kill();
                SSHP.Kill(Application);

                foreach (string Setting in Settings(Destination))
                {
                    File.Copy(Setting, Path.Combine(SMR.AppDataPath, SMR.AppName, Path.GetFileName(Setting)), true);
                }

                SSHP.Run(Application);
            }
            catch
            {
                //
            }
        }

        public static string[] Settings(string Path)
        {
            try
            {
                return Directory.GetFiles(Path, "*.json", SearchOption.TopDirectoryOnly);
            }
            catch
            {
                return Array.Empty<string>();
            }
        }
    }
}