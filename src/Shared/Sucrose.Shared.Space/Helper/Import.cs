using System.IO;
using SMR = Sucrose.Memory.Readonly;
using SSDEACT = Sucrose.Shared.Dependency.Enum.ArgumentCommandType;
using SSLHK = Sucrose.Shared.Live.Helper.Kill;
using SSSHP = Sucrose.Shared.Space.Helper.Processor;
using SSSMI = Sucrose.Shared.Space.Manage.Internal;

namespace Sucrose.Shared.Space.Helper
{
    internal static class Import
    {
        public static async Task Start(string Destination, string Application)
        {
            SSLHK.Stop();

            SSSHP.Kill(SMR.Undo);
            SSSHP.Kill(SMR.Portal);
            SSSHP.Kill(SMR.Update);
            SSSHP.Kill(SMR.Launcher);
            SSSHP.Kill(SMR.Property);
            SSSHP.Kill(SMR.Watchdog);
            SSSHP.Kill(SMR.Reportdog);
            SSSHP.Kill(SMR.Backgroundog);

            await Task.Delay(TimeSpan.FromSeconds(3));

            if (!Directory.Exists(Destination))
            {
                Directory.CreateDirectory(Destination);
            }

            if (!Directory.Exists(Path.Combine(SMR.AppDataPath, SMR.AppName, SMR.SettingFolder)))
            {
                Directory.CreateDirectory(Path.Combine(SMR.AppDataPath, SMR.AppName, SMR.SettingFolder));
            }

            foreach (string Setting in Settings(Destination))
            {
                File.Copy(Setting, Path.Combine(SMR.AppDataPath, SMR.AppName, SMR.SettingFolder, Path.GetFileName(Setting)), true);
            }

            await Task.Delay(TimeSpan.FromSeconds(1));

            SSSHP.Run(SSSMI.Portal, $"{SSDEACT.SystemSetting}");
            SSSHP.Run(Application);

            await Task.CompletedTask;
        }

        public static string[] Settings(string Destination)
        {
            try
            {
                return Directory.GetFiles(Destination, "*.json", SearchOption.TopDirectoryOnly);
            }
            catch
            {
                return Array.Empty<string>();
            }
        }
    }
}