using SMMRG = Sucrose.Memory.Manage.Readonly.General;

namespace Sucrose.Signal.Helper
{
    internal static class Reader
    {
        public static async Task<string> Read(string filePath)
        {
            try
            {
                using FileStream fileStream = new(filePath, FileMode.Open, FileAccess.Read, FileShare.None);
                using StreamReader reader = new(fileStream);

                return reader.ReadToEnd();
            }
            catch
            {
                try
                {
                    await Task.Delay(SMMRG.Randomise.Next(5, 50));

                    using FileStream fileStream = new(filePath, FileMode.Open, FileAccess.Read, FileShare.None);
                    using StreamReader reader = new(fileStream);

                    return reader.ReadToEnd();
                }
                catch
                {
                    try
                    {
                        await Task.Delay(SMMRG.Randomise.Next(5, 50));

                        using FileStream fileStream = new(filePath, FileMode.Open, FileAccess.Read, FileShare.None);
                        using StreamReader reader = new(fileStream);

                        return reader.ReadToEnd();
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