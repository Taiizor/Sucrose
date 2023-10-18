using SMR = Sucrose.Memory.Readonly;

namespace Sucrose.Signal.Helper
{
    internal static class Deleter
    {
        public static async Task Delete(string filePath)
        {
            try
            {
                if (File.Exists(filePath))
                {
                    File.Delete(filePath);
                }
            }
            catch
            {
                try
                {
                    await Task.Delay(SMR.Randomise.Next(5, 50));

                    if (File.Exists(filePath))
                    {
                        File.Delete(filePath);
                    }
                }
                catch
                {
                    try
                    {
                        await Task.Delay(SMR.Randomise.Next(5, 50));

                        if (File.Exists(filePath))
                        {
                            File.Delete(filePath);
                        }
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