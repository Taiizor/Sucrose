using System.IO;
using SMMRA = Sucrose.Memory.Manage.Readonly.App;
using SMMRP = Sucrose.Memory.Manage.Readonly.Path;
using SMR = Sucrose.Memory.Readonly;
using SSDEACT = Sucrose.Shared.Dependency.Enum.ArgumentCommandType;
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

            SSSHP.Kill(SMMRA.Undo);
            SSSHP.Kill(SMMRA.Portal);
            SSSHP.Kill(SMMRA.Update);
            SSSHP.Kill(SMMRA.Launcher);
            SSSHP.Kill(SMMRA.Property);
            SSSHP.Kill(SMMRA.Watchdog);
            SSSHP.Kill(SMMRA.Reportdog);
            SSSHP.Kill(SMMRA.Backgroundog);

            await Task.Delay(TimeSpan.FromSeconds(3));

            if (Directory.Exists(Path.Combine(SMMRP.ApplicationData, SMR.AppName, SMR.SettingFolder)))
            {
                foreach (string Setting in Settings(Path.Combine(SMMRP.ApplicationData, SMR.AppName, SMR.SettingFolder)))
                {
                    File.Delete(Setting);
                }
            }
            else
            {
                Directory.CreateDirectory(Path.Combine(SMMRP.ApplicationData, SMR.AppName, SMR.SettingFolder));
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