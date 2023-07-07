using System.IO;

namespace Sucrose.Manager.Helper
{
    internal static class Reader
    {
        public static string Read(string filePath)
        {
            try
            {
                //return File.ReadAllText(filePath);

                using FileStream fileStream = new(filePath, FileMode.Open, FileAccess.Read, FileShare.Read);
                using StreamReader reader = new(fileStream);
                return reader.ReadToEnd();
            }
            catch
            {
                return string.Empty;
            }
        }

        public static string ReadBasic(string filePath)
        {
            try
            {
                return File.ReadAllText(filePath);
            }
            catch
            {
                return string.Empty;
            }
        }
    }
}