using System.IO;
using SRMI = Sucrose.Reportdog.Manage.Internal;

namespace Sucrose.Backgroundog.Helper
{
    internal class Initialize : IDisposable
    {
        public void Start()
        {
            if (SRMI.FileWatcher == null)
            {
                if (!Directory.Exists(SRMI.Source))
                {
                    Directory.CreateDirectory(SRMI.Source);
                }

                SRMI.FileWatcher = new()
                {
                    NotifyFilter = NotifyFilters.LastWrite | NotifyFilters.FileName | NotifyFilters.DirectoryName,
                    Path = SRMI.Source,
                    Filter = "*.*"
                };

                SRMI.FileWatcher.Created += async (s, e) =>
                {
                    if (File.Exists(e.FullPath))
                    {
                        await Task.Delay(50);

                        //CreatedEventHandler?.Invoke(s, e);
                    }
                };

                SRMI.FileWatcher.EnableRaisingEvents = true;
            }
        }

        public void Stop()
        {
            if (SRMI.FileWatcher != null)
            {
                SRMI.FileWatcher.EnableRaisingEvents = false;
                SRMI.FileWatcher.Dispose();
            }
        }

        public void Dispose()
        {
            GC.Collect();
            GC.SuppressFinalize(this);
        }
    }
}