using System.IO;
using SMR = Sucrose.Memory.Readonly;
using SSHC = Sucrose.Space.Helper.Command;
using SSHL = Sucrose.Space.Helper.Live;

namespace Sucrose.Space.Helper
{
    internal static class Import
    {
        public static void Start(string Destination, string Application)
        {
            try
            {
                SSHL.Kill();
                SSHC.Kill(Application);

                foreach (string Setting in Settings(Destination))
                {
                    File.Copy(Setting, Path.Combine(SMR.AppDataPath, SMR.AppName, Path.GetFileName(Setting)), true);
                }

                SSHC.Run(Application);
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