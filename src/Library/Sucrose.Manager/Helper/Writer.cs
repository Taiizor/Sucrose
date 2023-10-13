using SMHC = Sucrose.Manager.Helper.Cleaner;
using SMR = Sucrose.Memory.Readonly;

namespace Sucrose.Manager.Helper
{
    internal static class Writer
    {
        public static async void Write(string filePath, string fileContent)
        {
            FileMode fileMode = File.Exists(filePath) ? FileMode.Truncate : FileMode.CreateNew;

            try
            {
                using FileStream fileStream = new(filePath, fileMode, FileAccess.Write, FileShare.None);
                using StreamWriter writer = new(fileStream);

                writer.Write(SMHC.Clean(fileContent));
            }
            catch
            {
                try
                {
                    await Task.Delay(SMR.Randomise.Next(5, 50));

                    using FileStream fileStream = new(filePath, fileMode, FileAccess.Write, FileShare.None);
                    using StreamWriter writer = new(fileStream);

                    writer.Write(SMHC.Clean(fileContent));
                }
                catch
                {
                    try
                    {
                        await Task.Delay(SMR.Randomise.Next(5, 50));

                        using FileStream fileStream = new(filePath, fileMode, FileAccess.Write, FileShare.None);
                        using StreamWriter writer = new(fileStream);

                        writer.Write(SMHC.Clean(fileContent));
                    }
                    catch
                    {
                        //
                    }
                }
            }
        }

        public static async void WriteBasic(string filePath, string fileContent)
        {
            try
            {
                using StreamWriter writer = File.AppendText(filePath);

                writer.WriteLine(SMHC.Clean(fileContent));
            }
            catch
            {
                try
                {
                    await Task.Delay(SMR.Randomise.Next(5, 50));

                    using StreamWriter writer = File.AppendText(filePath);

                    writer.WriteLine(SMHC.Clean(fileContent));
                }
                catch
                {
                    try
                    {
                        await Task.Delay(SMR.Randomise.Next(5, 50));

                        using StreamWriter writer = File.AppendText(filePath);

                        writer.WriteLine(SMHC.Clean(fileContent));
                    }
                    catch
                    {
                        //
                    }
                }
            }
        }

        public static async void WriteStable(string filePath, string fileContent)
        {
            try
            {
                File.WriteAllText(filePath, SMHC.Clean(fileContent));
            }
            catch
            {
                try
                {
                    await Task.Delay(SMR.Randomise.Next(5, 50));

                    File.WriteAllText(filePath, SMHC.Clean(fileContent));
                }
                catch
                {
                    try
                    {
                        await Task.Delay(SMR.Randomise.Next(5, 50));

                        File.WriteAllText(filePath, SMHC.Clean(fileContent));
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