namespace Sucrose.Signal.Helper
{
    internal static class Reader
    {
        public static string Read(string filePath)
        {
            using Mutex Mutex = new(false, Path.GetFileName(filePath));

            try
            {
                try
                {
                    Mutex.WaitOne();
                }
                catch { }

                using FileStream fileStream = new(filePath, FileMode.Open, FileAccess.Read, FileShare.None);
                using StreamReader reader = new(fileStream);

                return reader.ReadToEnd();
            }
            finally
            {
                Mutex.ReleaseMutex();
            }
        }
    }
}