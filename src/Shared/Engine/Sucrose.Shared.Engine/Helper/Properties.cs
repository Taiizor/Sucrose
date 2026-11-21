using Newtonsoft.Json;
using System.IO;
using SESMIEN = Sucrose.Shared.Engine.Manage.Internal.ExecuteNormal;
using SESMIET = Sucrose.Shared.Engine.Manage.Internal.ExecuteTask;
using SSEMI = Sucrose.Shared.Engine.Manage.Internal;
using SSSHF = Sucrose.Shared.Space.Helper.Filing;
using SSTMCM = Sucrose.Shared.Theme.Model.ControlModel;
using SSTMFDDM = Sucrose.Shared.Theme.Model.FileDropDownModel;
using SSWEW = Sucrose.Shared.Watchdog.Extension.Watch;
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

            FileWatcher = new(Path.GetDirectoryName(Source))
            {
                Filter = Path.GetFileName(Source),

                NotifyFilter = NotifyFilters.LastWrite | NotifyFilters.FileName | NotifyFilters.DirectoryName | NotifyFilters.CreationTime
            };
        }

        private static void WatcherTimer(string Source)
        {
            Timer Deletion = new(3000);

            Deletion.Elapsed += (s, e) =>
            {
                try
                {
                    SSSHF.Delete(Source);
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
                    if (Record.Contains(Path.GetFileName(Source)[1..]))
                    {
                        SSSHF.Delete(Record);
                    }
                }
            }
            catch { }
        }

        public static async void ExecuteTask(SESMIET Function)
        {
            try
            {
                if (SSEMI.Initialized)
                {
                    SESMIEN AdaptedFunction = new(async (Script) =>
                    {
                        try
                        {
                            await Function(Script);
                        }
                        catch (Exception Exception)
                        {
                            await SSWEW.Watch_CatchException(Exception);
                        }
                    });

                    ExecuteNormal(AdaptedFunction);
                }
            }
            catch (Exception Exception)
            {
                await SSWEW.Watch_CatchException(Exception);
            }
        }

        private static string[] ListFiles(SSTMFDDM Control)
        {
            try
            {
                if (string.IsNullOrWhiteSpace(SSEMI.LibraryLocation) || string.IsNullOrWhiteSpace(SSEMI.LibrarySelected) || string.IsNullOrWhiteSpace(Control.Folder) || string.IsNullOrWhiteSpace(Control.Filter))
                {
                    return null;
                }

                string Root = Path.GetFullPath(Path.Combine(SSEMI.LibraryLocation, SSEMI.LibrarySelected)).TrimEnd(Path.DirectorySeparatorChar, Path.AltDirectorySeparatorChar);
                string Folder = Path.GetFullPath(Path.Combine(Root, Control.Folder)).TrimEnd(Path.DirectorySeparatorChar, Path.AltDirectorySeparatorChar);

                // Only list folders inside the wallpaper itself; a rooted or ".." folder must not expose other directories to the page
                if (!Folder.Equals(Root, StringComparison.OrdinalIgnoreCase) && !Folder.StartsWith(Root + Path.DirectorySeparatorChar, StringComparison.OrdinalIgnoreCase))
                {
                    return [];
                }

                if (!Directory.Exists(Folder))
                {
                    return [];
                }

                string[] Extensions = Control.Filter.Replace("*", string.Empty).Split('|', StringSplitOptions.RemoveEmptyEntries | StringSplitOptions.TrimEntries);

                return Directory.GetFiles(Folder, "*.*", SearchOption.TopDirectoryOnly)
                    .Where(Record => Extensions.Any(Extension => Record.EndsWith(Extension, StringComparison.OrdinalIgnoreCase)))
                    .Select(Path.GetFileName)
                    .OrderBy(Record => Record, StringComparer.OrdinalIgnoreCase)
                    .ToArray();
            }
            catch
            {
                return null;
            }
        }

        public static async void ExecuteNormal(SESMIEN Function)
        {
            try
            {
                if (SSEMI.Initialized)
                {
                    if (SSEMI.Properties.PropertyList.Any())
                    {
                        foreach (KeyValuePair<string, SSTMCM> Pair in SSEMI.Properties.PropertyList)
                        {
                            string Key = Pair.Key;
                            object Value = Pair.Value;

                            if (Value is SSTMFDDM FileDropDown)
                            {
                                FileDropDown.Files = ListFiles(FileDropDown);
                            }

                            string Script = JsonConvert.SerializeObject(Value, Formatting.Indented);

                            try
                            {
                                Function(string.Format(SSEMI.Properties.PropertyListener, Key, Script));
                            }
                            catch (Exception Exception)
                            {
                                await SSWEW.Watch_CatchException(Exception);
                            }
                        }
                    }
                }
            }
            catch (Exception Exception)
            {
                await SSWEW.Watch_CatchException(Exception);
            }
        }
    }
}