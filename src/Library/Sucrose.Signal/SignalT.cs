﻿using Newtonsoft.Json;
using SMMRP = Sucrose.Memory.Manage.Readonly.Path;
using SMR = Sucrose.Memory.Readonly;
using SSHD = Sucrose.Signal.Helper.Deleter;
using SSHR = Sucrose.Signal.Helper.Reader;
using SSHW = Sucrose.Signal.Helper.Writer;
using Timer = System.Timers.Timer;

namespace Sucrose.Signal
{
    public class SignalT(string Name)
    {
        private readonly JsonSerializerSettings SerializerSettings = new() { TypeNameHandling = TypeNameHandling.None, Formatting = Formatting.None };
        private readonly string Source = Path.Combine(SMMRP.ApplicationData, SMR.AppName, SMR.CacheFolder, SMR.SignalT);
        private FileSystemWatcher FileWatcher;

        public FileSystemEventHandler CreatedEventHandler;
        //public FileSystemEventHandler ChangedEventHandler;
        //public FileSystemEventHandler DeletedEventHandler;
        //public RenamedEventHandler RenamedEventHandler;

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

            //FileWatcher.Changed += (s, e) =>
            //{
            //    if (FileCheck(e.FullPath))
            //    {
            //        ChangedEventHandler?.Invoke(s, e);
            //    }
            //};

            //FileWatcher.Deleted += (s, e) =>
            //{
            //    if (FileCheck(e.FullPath))
            //    {
            //        DeletedEventHandler?.Invoke(s, e);
            //    }
            //};

            //FileWatcher.Renamed += (s, e) =>
            //{
            //    if (FileCheck(e.FullPath))
            //    {
            //        RenamedEventHandler?.Invoke(s, e);
            //    }
            //};

            FileWatcher.EnableRaisingEvents = true;
        }

        public void FileSave<T>(T Data)
        {
            string Destination = Path.Combine(Source, $"{Path.GetFileNameWithoutExtension(Name)}-{Guid.NewGuid()}{Path.GetExtension(Name)}");

            SSHW.Write(Destination, JsonConvert.SerializeObject(Data, SerializerSettings));
        }

        public string FileName(string Source)
        {
            return Path.GetFileName(Source);
        }

        public async void FileDelete(string Source)
        {
            await SSHD.Delete(Source);
        }

        public async Task<T> FileRead<T>(string Source, T Default, bool Delete = true)
        {
            try
            {
                await Task.Delay(SMR.Randomise.Next(5, 50));

                string Data = SSHR.Read(Source).Result;

                if (Delete)
                {
                    DeletionTimer(Source);
                }

                return JsonConvert.DeserializeObject<T>(Data, SerializerSettings);
            }
            catch
            {
                return Default;
            }
        }

        public string FileRead(string Source, string Default, bool Delete = true)
        {
            try
            {
                string Data = SSHR.Read(Source).Result;

                if (Delete)
                {
                    DeletionTimer(Source);
                }

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

        private void DeletionTimer(string Source)
        {
            Timer Deletion = new(3000);

            Deletion.Elapsed += (s, e) =>
            {
                FileDelete(Source);

                Deletion.Stop();
                Deletion.Dispose();
            };

            Deletion.AutoReset = false;

            Deletion.Start();
        }
    }
}