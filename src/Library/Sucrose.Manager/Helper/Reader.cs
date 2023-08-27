using SMHC = Sucrose.Manager.Helper.Cleaner;

namespace Sucrose.Manager.Helper
{
    internal static class Reader
    {
        public static string Read(string filePath)
        {
            try
            {
                using FileStream fileStream = new(filePath, FileMode.Open, FileAccess.Read, FileShare.None);
                using StreamReader reader = new(fileStream);

                return SMHC.Clean(reader.ReadToEnd());
            }
            catch
            {
                try
                {
                    using FileStream fileStream = new(filePath, FileMode.Open, FileAccess.Read, FileShare.None);
                    using StreamReader reader = new(fileStream);

                    return SMHC.Clean(reader.ReadToEnd());
                }
                catch
                {
                    return string.Empty;
                }
            }
        }

        public static string ReadBasic(string filePath)
        {
            try
            {
                return SMHC.Clean(File.ReadAllText(filePath));
            }
            catch
            {
                try
                {
                    return SMHC.Clean(File.ReadAllText(filePath));
                }
                catch
                {
                    return string.Empty;
                }
            }
        }
    }
}