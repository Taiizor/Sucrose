using System.Text.Json;
using SMR = Sucrose.Memory.Readonly;
using SSHR = Sucrose.Signal.Helper.Reader;
using SSHW = Sucrose.Signal.Helper.Writer;

namespace Sucrose.Signal
{
    public class SignalT(string Name)
    {
        private readonly string Source = Path.Combine(SMR.AppDataPath, SMR.AppName, SMR.CacheFolder, SMR.Signal);
        private readonly JsonSerializerOptions Options = new() { WriteIndented = true };
        private readonly Random Random = new();
        private FileSystemWatcher FileWatcher;

        public FileSystemEventHandler CreatedEventHandler;
        public FileSystemEventHandler ChangedEventHandler;
        public FileSystemEventHandler DeletedEventHandler;
        public RenamedEventHandler RenamedEventHandler;

        public void StopChannel()
        {
            if (FileWatcher != null)
            {
                FileWatcher.EnableRaisingEvents = false;
                FileWatcher.Dispose();
            }
        }

        public void StartChannel(FileSystemEventHandler Handler)
        {
            CreatedEventHandler += Handler;

            if (Directory.Exists(Source))
            {
                string[] Files = Directory.GetFiles(Source, "*.*", SearchOption.TopDirectoryOnly);

                foreach (string Record in Files)
                {
                    if (FileCheck(Record))
                    {
                        File.Delete(Record);
                    }
                }
            }
            else
            {
                Directory.CreateDirectory(Source);
            }

            FileWatcher = new()
            {
                NotifyFilter = NotifyFilters.LastWrite | NotifyFilters.FileName | NotifyFilters.DirectoryName,
                Filter = "*.*",
                Path = Source
            };

            FileWatcher.Created += (s, e) =>
            {
                if (FileCheck(e.FullPath))
                {
                    CreatedEventHandler?.Invoke(s, e);
                }
            };

            FileWatcher.Changed += (s, e) =>
            {
                if (FileCheck(e.FullPath))
                {
                    ChangedEventHandler?.Invoke(s, e);
                }
            };

            FileWatcher.Deleted += (s, e) =>
            {
                if (FileCheck(e.FullPath))
                {
                    DeletedEventHandler?.Invoke(s, e);
                }
            };

            FileWatcher.Renamed += (s, e) =>
            {
                if (FileCheck(e.FullPath))
                {
                    RenamedEventHandler?.Invoke(s, e);
                }
            };

            FileWatcher.EnableRaisingEvents = true;
        }

        public void FileSave<T>(T Data)
        {
            string Destination = Path.Combine(Source, Name);

            while (File.Exists(Destination))
            {
                Destination = Path.Combine(Source, $"{Path.GetFileNameWithoutExtension(Name)}-{Random.Next(0, int.MaxValue)}{Path.GetExtension(Name)}");
            }

            SSHW.Write(Destination, JsonSerializer.Serialize(Data, Options));
        }

        public string FileName(string Source)
        {
            return Path.GetFileName(Source);
        }

        public T FileRead<T>(string Source, T Default)
        {
            try
            {
                string Data = SSHR.Read(Source).Result;

                return JsonSerializer.Deserialize<T>(Data, Options);
            }
            catch
            {
                return Default;
            }
        }

        public string FileRead(string Source, string Default)
        {
            try
            {
                string Data = SSHR.Read(Source).Result;

                return Data;
            }
            catch
            {
                return Default;
            }
        }

        private bool FileCheck(string Source)
        {
            if (FileName(Source).Contains(Path.GetFileNameWithoutExtension(Name)) || FileName(Source) == Name)
            {
                return true;
            }
            else
            {
                return false;
            }
        }
    }
}