using System.IO;
using SMR = Sucrose.Memory.Readonly;
using SSDEACT = Sucrose.Shared.Dependency.Enum.ArgumentCommandsType;
using SSLHK = Sucrose.Shared.Live.Helper.Kill;
using SSSHP = Sucrose.Shared.Space.Helper.Processor;
using SSSMI = Sucrose.Shared.Space.Manage.Internal;

namespace Sucrose.Shared.Space.Helper
{
    internal static class Reset
    {
        public static async Task Start(string Application)
        {
            SSLHK.Stop();

            SSSHP.Kill(SMR.Undo);
            SSSHP.Kill(SMR.Portal);
            SSSHP.Kill(SMR.Update);
            SSSHP.Kill(SMR.Launcher);
            SSSHP.Kill(SMR.Property);
            SSSHP.Kill(SMR.Watchdog);
            SSSHP.Kill(SMR.Backgroundog);

            await Task.Delay(TimeSpan.FromSeconds(3));

            if (Directory.Exists(Path.Combine(SMR.AppDataPath, SMR.AppName, SMR.SettingFolder)))
            {
                foreach (string Setting in Settings(Path.Combine(SMR.AppDataPath, SMR.AppName, SMR.SettingFolder)))
                {
                    File.Delete(Setting);
                }
            }
            else
            {
                Directory.CreateDirectory(Path.Combine(SMR.AppDataPath, SMR.AppName, SMR.SettingFolder));
            }

            await Task.Delay(TimeSpan.FromSeconds(1));

            SSSHP.Run(Application);
            SSSHP.Run(SSSMI.Portal, $"{SSDEACT.SystemSetting}");

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