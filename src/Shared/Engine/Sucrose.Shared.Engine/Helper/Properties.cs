using Newtonsoft.Json;
using System.IO;
using SESMIEN = Sucrose.Shared.Engine.Manage.Internal.ExecuteNormal;
using SESMIET = Sucrose.Shared.Engine.Manage.Internal.ExecuteTask;
using SSEMI = Sucrose.Shared.Engine.Manage.Internal;
using SSTHP = Sucrose.Shared.Theme.Helper.Properties;

namespace Sucrose.Shared.Engine.Helper
{
    internal static class Properties
    {
        private static FileSystemWatcher FileWatcher;
        public static FileSystemEventHandler ChangedEventHandler;

        public static void StopWatcher()
        {
            if (SSEMI.PropertiesWatcher && FileWatcher != null)
            {
                FileWatcher.EnableRaisingEvents = false;
                SSEMI.PropertiesWatcher = true;
                FileWatcher.Dispose();
            }
        }

        public static void StartWatcher()
        {
            if (SSEMI.PropertiesWatcher)
            {
                SSEMI.PropertiesWatcher = false;

                FileWatcher.EnableRaisingEvents = true;

                FileWatcher.Changed += async (s, e) =>
                {
                    if (SSEMI.Initialized && !SSEMI.PausePerformance && File.Exists(e.FullPath))
                    {
                        await Task.Delay(50);

                        ChangedEventHandler?.Invoke(s, e);
                    }
                };
            }
        }

        public static void Watcher(string File)
        {
            FileWatcher = new(Path.GetDirectoryName(File));

            FileWatcher.Filter = Path.GetFileName(File);

            FileWatcher.NotifyFilter = NotifyFilters.LastWrite;
        }

        private static void PropertiesFile_Changed(object sender, FileSystemEventArgs e)
        {
            if (SSEMI.Initialized)
            {
                if (File.Exists(e.FullPath))
                {
                    SSEMI.Properties = SSTHP.ReadJson(e.FullPath);
                }
            }
        }

        public static void ExecuteNormal(SESMIEN Function)
        {
            if (SSEMI.Initialized)
            {
                if (SSEMI.Properties.PropertyList.Any())
                {
                    foreach (KeyValuePair<string, object> Pair in SSEMI.Properties.PropertyList)
                    {
                        string Key = Pair.Key;
                        object Value = Pair.Value;

                        string Script = JsonConvert.SerializeObject(Value, Formatting.Indented);

                        Function(string.Format(SSEMI.Properties.PropertyListener, Key, Script));
                    }
                }
            }
        }

        public static void ExecuteTask(SESMIET Function)
        {
            if (SSEMI.Initialized)
            {
                SESMIEN AdaptedFunction = new(async (Script) =>
                {
                    await Function(Script);
                });

                ExecuteNormal(AdaptedFunction);
            }
        }
    }
}