using System.IO;

namespace Sucrose.Manager.Helper
{
    internal static class Writer
    {
        public static void Write(string filePath, string fileContent)
        {
            try
            {
                //File.WriteAllText(filePath, fileContent);

                using FileStream fileStream = new(filePath, FileMode.OpenOrCreate, FileAccess.Write, FileShare.Read);
                using StreamWriter writer = new(fileStream);
                writer.Write(fileContent);
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
                using StreamWriter writer = File.AppendText(filePath);
                writer.WriteLine(fileContent);
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
                //File.WriteAllText(filePath, fileContent);

                using FileStream fileStream = new(filePath, FileMode.OpenOrCreate, FileAccess.Write, FileShare.Read);
                using StreamWriter writer = new(fileStream);
                writer.WriteLine(fileContent);
            }
            catch
            {
                //
            }
        }
    }
}