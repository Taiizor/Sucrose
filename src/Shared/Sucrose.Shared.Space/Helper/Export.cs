using System.IO;
using SMR = Sucrose.Memory.Readonly;

namespace Sucrose.Shared.Space.Helper
{
    internal static class Export
    {
        public static void Start(string Destination)
        {
            try
            {
                foreach (string Setting in Settings(Path.Combine(SMR.AppDataPath, SMR.AppName, SMR.SettingFolder)))
                {
                    File.Copy(Setting, Destination, true);
                }
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