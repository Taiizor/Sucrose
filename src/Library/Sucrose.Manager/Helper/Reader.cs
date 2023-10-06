using SMHC = Sucrose.Manager.Helper.Cleaner;
using SMR = Sucrose.Memory.Readonly;

namespace Sucrose.Manager.Helper
{
    internal static class Reader
    {
        public static async Task<string> Read(string filePath)
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
                    await Task.Delay(SMR.Randomise.Next(10, 100));

                    using FileStream fileStream = new(filePath, FileMode.Open, FileAccess.Read, FileShare.None);
                    using StreamReader reader = new(fileStream);

                    return SMHC.Clean(reader.ReadToEnd());
                }
                catch
                {
                    try
                    {
                        await Task.Delay(SMR.Randomise.Next(10, 100));

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
        }

        public static async Task<string> ReadBasic(string filePath)
        {
            try
            {
                return SMHC.Clean(File.ReadAllText(filePath));
            }
            catch
            {
                try
                {
                    await Task.Delay(SMR.Randomise.Next(10, 100));

                    return SMHC.Clean(File.ReadAllText(filePath));
                }
                catch
                {
                    try
                    {
                        await Task.Delay(SMR.Randomise.Next(10, 100));

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
}