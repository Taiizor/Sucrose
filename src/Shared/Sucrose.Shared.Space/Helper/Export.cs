﻿using System.IO;
using SMR = Sucrose.Memory.Readonly;
using SSDEACT = Sucrose.Shared.Dependency.Enum.ArgumentCommandsType;
using SSLHK = Sucrose.Shared.Live.Helper.Kill;
using SSSHP = Sucrose.Shared.Space.Helper.Processor;
using SSSMI = Sucrose.Shared.Space.Manage.Internal;

namespace Sucrose.Shared.Space.Helper
{
    internal static class Export
    {
        public static async Task Start(string Destination, string Application)
        {
            SSLHK.Stop();
            SSSHP.Kill(SMR.Portal);
            SSSHP.Kill(SMR.Update);
            SSSHP.Kill(SMR.Launcher);
            SSSHP.Kill(SMR.Backgroundog);

            await Task.Delay(TimeSpan.FromSeconds(1));

            foreach (string Setting in Settings(Path.Combine(SMR.AppDataPath, SMR.AppName, SMR.SettingFolder)))
            {
                File.Copy(Setting, Path.Combine(Destination, Path.GetFileName(Setting)), true);
            }

            await Task.Delay(TimeSpan.FromSeconds(1));

            SSSHP.Run(Application);
            SSSHP.Run(SSSMI.Portal, $"{SSDEACT.SystemSetting}");

            await Task.CompletedTask;
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