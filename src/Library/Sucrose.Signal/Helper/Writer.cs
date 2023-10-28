using SMR = Sucrose.Memory.Readonly;

namespace Sucrose.Signal.Helper
{
    internal static class Writer
    {
        public static async void Write(string filePath, string fileContent)
        {
            try
            {
                using FileStream fileStream = new(filePath, FileMode.CreateNew, FileAccess.Write, FileShare.None);
                using StreamWriter writer = new(fileStream);

                writer.Write(fileContent);
            }
            catch
            {
                try
                {
                    await Task.Delay(SMR.Randomise.Next(5, 50));

                    using FileStream fileStream = new(filePath, FileMode.CreateNew, FileAccess.Write, FileShare.None);
                    using StreamWriter writer = new(fileStream);

                    writer.Write(fileContent);
                }
                catch
                {
                    try
                    {
                        await Task.Delay(SMR.Randomise.Next(5, 50));

                        using FileStream fileStream = new(filePath, FileMode.CreateNew, FileAccess.Write, FileShare.None);
                        using StreamWriter writer = new(fileStream);

                        writer.Write(fileContent);
                    }
                    catch
                    {
                        //
                    }
                }
            }
        }
    }
}