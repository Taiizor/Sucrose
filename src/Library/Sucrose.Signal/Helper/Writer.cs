namespace Sucrose.Signal.Helper
{
    internal static class Writer
    {
        public static void Write(string filePath, string fileContent)
        {
            using Mutex Mutex = new(false, Path.GetFileName(filePath));

            try
            {
                try
                {
                    Mutex.WaitOne();
                }
                catch { }

                using FileStream fileStream = new(filePath, FileMode.CreateNew, FileAccess.Write, FileShare.None);
                using StreamWriter writer = new(fileStream);

                writer.Write(fileContent);
            }
            finally
            {
                Mutex.ReleaseMutex();
            }
        }
    }
}