using Newtonsoft.Json;
using System.IO;
using SESMIEN = Sucrose.Shared.Engine.Manage.Internal.ExecuteNormal;
using SESMIET = Sucrose.Shared.Engine.Manage.Internal.ExecuteTask;
using SSEMI = Sucrose.Shared.Engine.Manage.Internal;
using SSTMCM = Sucrose.Shared.Theme.Model.ControlModel;
using Timer = System.Timers.Timer;

namespace Sucrose.Shared.Engine.Helper
{
    internal static class Properties
    {
        private static FileSystemWatcher FileWatcher;
        public static FileSystemEventHandler CreatedEventHandler;

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

                FileWatcher.Created += async (s, e) =>
                {
                    if (SSEMI.Initialized && !SSEMI.PausePerformance && File.Exists(e.FullPath))
                    {
                        await Task.Delay(50);

                        WatcherTimer(e.FullPath);

                        CreatedEventHandler?.Invoke(s, e);
                    }
                };
            }
        }

        public static void Watcher(string Source)
        {
            WatcherCheck(Source);

            FileWatcher = new(Path.GetDirectoryName(Source));

            FileWatcher.Filter = Path.GetFileName(Source);

            FileWatcher.NotifyFilter = NotifyFilters.LastWrite | NotifyFilters.FileName | NotifyFilters.DirectoryName | NotifyFilters.CreationTime;
        }

        private static void WatcherTimer(string Source)
        {
            Timer Deletion = new(3000);

            Deletion.Elapsed += (s, e) =>
            {
                try
                {
                    File.Delete(Source);
                }
                catch { }

                Deletion.Stop();
                Deletion.Dispose();
            };

            Deletion.AutoReset = false;

            Deletion.Start();
        }

        private static void WatcherCheck(string Source)
        {
            try
            {
                string[] Files = Directory.GetFiles(Path.GetDirectoryName(Source), "*.*", SearchOption.TopDirectoryOnly);

                foreach (string Record in Files)
                {
#if NET48_OR_GREATER
                    if (Record.Contains(Path.GetFileName(Source).Substring(1)))
#else
                    if (Record.Contains(Path.GetFileName(Source)[1..]))
#endif
                    {
                        File.Delete(Record);
                    }
                }
            }
            catch { }
        }

        public static void ExecuteNormal(SESMIEN Function)
        {
            if (SSEMI.Initialized)
            {
                if (SSEMI.Properties.PropertyList.Any())
                {
                    foreach (KeyValuePair<string, SSTMCM> Pair in SSEMI.Properties.PropertyList)
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