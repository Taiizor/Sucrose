using SMHC = Sucrose.Manager.Helper.Cleaner;

namespace Sucrose.Manager.Helper
{
    internal static class Writer
    {
        public static void Write(string filePath, string fileContent)
        {
            try
            {
                using Mutex Mutex = new(false, Path.GetFileName(filePath));

                try
                {
                    Mutex.WaitOne();

                    using FileStream fileStream = new(filePath, FileMode.OpenOrCreate, FileAccess.Write, FileShare.None);
                    using StreamWriter writer = new(fileStream);
                    writer.Write(SMHC.Clean(fileContent));
                }
                finally
                {
                    Mutex.ReleaseMutex();
                }
            }
            catch
            {
                //
            }
        }

        public static void WriteBasic(string filePath, string fileContent)
        {
            try
            {
                using Mutex Mutex = new(false, Path.GetFileName(filePath));

                try
                {
                    Mutex.WaitOne();

                    using StreamWriter writer = File.AppendText(filePath);
                    writer.WriteLine(SMHC.Clean(fileContent));
                }
                finally
                {
                    Mutex.ReleaseMutex();
                }
            }
            catch
            {
                //
            }
        }

        public static void WriteStable(string filePath, string fileContent)
        {
            try
            {
                using Mutex Mutex = new(false, Path.GetFileName(filePath));

                try
                {
                    Mutex.WaitOne();

                    File.WriteAllText(filePath, SMHC.Clean(fileContent));
                }
                finally
                {
                    Mutex.ReleaseMutex();
                }
            }
            catch
            {
                //
            }
        }
    }
}