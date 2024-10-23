using SMHC = Sucrose.Manager.Helper.Cleaner;

namespace Sucrose.Manager.Helper
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

                return SMHC.Clean(reader.ReadToEnd());
            }
            finally
            {
                Mutex.ReleaseMutex();
            }
        }

        public static string ReadBasic(string filePath)
        {
            using Mutex Mutex = new(false, Path.GetFileName(filePath));

            try
            {
                try
                {
                    Mutex.WaitOne();
                }
                catch { }

                return SMHC.Clean(File.ReadAllText(filePath));
            }
            finally
            {
                Mutex.ReleaseMutex();
            }
        }
    }
}