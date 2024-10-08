using System.IO;
using SSSHU = Sucrose.Shared.Space.Helper.Unique;

namespace Sucrose.Shared.Space.Helper
{
    internal static class Filing
    {
        public static string Read(string Source)
        {
            using Mutex Mutex = new(false, SSSHU.GenerateText(Source));

            try
            {
                try
                {
                    Mutex.WaitOne();
                }
                catch { }

                return File.ReadAllText(Source);
            }
            finally
            {
                Mutex.ReleaseMutex();
            }
        }

        public static string ReadStream(string Source)
        {
            using Mutex Mutex = new(false, SSSHU.GenerateText(Source));

            try
            {
                try
                {
                    Mutex.WaitOne();
                }
                catch { }

                using FileStream Stream = new(Source, FileMode.Open, FileAccess.Read, FileShare.None);

                using StreamReader Reader = new(Stream);

                return Reader.ReadToEnd();
            }
            finally
            {
                Mutex.ReleaseMutex();
            }
        }

        public static void Write(string Source, string Content)
        {
            using Mutex Mutex = new(false, SSSHU.GenerateText(Source));
            try
            {
                try
                {
                    Mutex.WaitOne();
                }
                catch { }

                File.WriteAllText(Source, Content);
            }
            finally
            {
                Mutex.ReleaseMutex();
            }
        }

        public static void WriteStream(string Source, string Content)
        {
            using Mutex Mutex = new(false, SSSHU.GenerateText(Source));

            try
            {
                try
                {
                    Mutex.WaitOne();
                }
                catch { }

                using FileStream Stream = new(Source, FileMode.OpenOrCreate, FileAccess.Write, FileShare.None);

                using StreamWriter Writer = new(Stream);

                Writer.Write(Content);
            }
            finally
            {
                Mutex.ReleaseMutex();
            }
        }
    }
}