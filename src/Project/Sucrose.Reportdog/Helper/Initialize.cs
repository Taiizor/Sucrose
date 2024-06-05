using System.IO;
using SRMI = Sucrose.Reportdog.Manage.Internal;

namespace Sucrose.Backgroundog.Helper
{
    internal class Initialize : IDisposable
    {
        public void Start()
        {
            if (SRMI.Watcher == null)
            {
                if (!Directory.Exists(SRMI.Source))
                {
                    Directory.CreateDirectory(SRMI.Source);
                }

                SRMI.Watcher = new()
                {
                    NotifyFilter = NotifyFilters.LastWrite | NotifyFilters.FileName | NotifyFilters.DirectoryName,
                    Path = SRMI.Source,
                    Filter = "*.*"
                };

                SRMI.Watcher.Created += async (s, e) =>
                {
                    if (File.Exists(e.FullPath))
                    {
                        await Task.Delay(50);

                        //CreatedEventHandler?.Invoke(s, e);
                    }
                };

                SRMI.Watcher.EnableRaisingEvents = true;
            }
        }

        public void Stop()
        {
            if (SRMI.Watcher != null)
            {
                SRMI.Watcher.EnableRaisingEvents = false;
                SRMI.Watcher.Dispose();
            }
        }

        public void Dispose()
        {
            GC.Collect();
            GC.SuppressFinalize(this);
        }
    }
}